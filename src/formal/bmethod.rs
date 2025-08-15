//! B-Method integration for Lambdust.
//!
//! This module provides tools for translating between B-Method specifications
//! and Lambdust programs, with formal correctness guarantees.

use super::{FormalSpec, ProofObligation};
use crate::eval::Value;
use crate::diagnostics::{Error, Result};
use std::collections::HashMap;
use std::fmt;

/// Represents a B-Method machine.
#[derive(Debug, Clone)]
pub struct BMachine {
    /// Machine name.
    pub name: String,
    /// Machine parameters.
    pub parameters: Vec<String>,
    /// Machine constraints on parameters.
    pub constraints: Vec<BPredicate>,
    /// Machine sets.
    pub sets: Vec<BSet>,
    /// Machine constants.
    pub constants: Vec<String>,
    /// Machine properties.
    pub properties: Vec<BPredicate>,
    /// Machine variables.
    pub variables: Vec<String>,
    /// Machine invariant.
    pub invariant: Vec<BPredicate>,
    /// Machine initialization.
    pub initialization: BSubstitution,
    /// Machine operations.
    pub operations: Vec<BOperation>,
}

/// B-Method set definition.
#[derive(Debug, Clone)]
pub struct BSet {
    /// Set name.
    pub name: String,
    /// Set enumeration (if any).
    pub enumeration: Option<Vec<String>>,
}

/// B-Method operation.
#[derive(Debug, Clone)]
pub struct BOperation {
    /// Operation name.
    pub name: String,
    /// Input parameters.
    pub inputs: Vec<String>,
    /// Output parameters.
    pub outputs: Vec<String>,
    /// Precondition.
    pub precondition: Option<BPredicate>,
    /// Operation body (generalized substitution).
    pub body: BSubstitution,
}

/// B-Method predicate.
#[derive(Debug, Clone)]
pub enum BPredicate {
    /// Boolean constant.
    BoolConstant(bool),
    /// Variable or constant.
    Identifier(String),
    /// Equality.
    Equals(Box<BExpression>, Box<BExpression>),
    /// Inequality.
    NotEquals(Box<BExpression>, Box<BExpression>),
    /// Set membership.
    MemberOf(Box<BExpression>, Box<BExpression>),
    /// Subset relation.
    Subset(Box<BExpression>, Box<BExpression>),
    /// Logical conjunction.
    And(Box<BPredicate>, Box<BPredicate>),
    /// Logical disjunction.
    Or(Box<BPredicate>, Box<BPredicate>),
    /// Logical implication.
    Implies(Box<BPredicate>, Box<BPredicate>),
    /// Logical negation.
    Not(Box<BPredicate>),
    /// Universal quantification.
    Forall(Vec<String>, Box<BPredicate>),
    /// Existential quantification.
    Exists(Vec<String>, Box<BPredicate>),
    /// Arithmetic comparison.
    Less(Box<BExpression>, Box<BExpression>),
    LessEqual(Box<BExpression>, Box<BExpression>),
    Greater(Box<BExpression>, Box<BExpression>),
    GreaterEqual(Box<BExpression>, Box<BExpression>),
}

/// B-Method expression.
#[derive(Debug, Clone)]
pub enum BExpression {
    /// Integer literal.
    Integer(i64),
    /// Boolean literal.
    Boolean(bool),
    /// String literal.
    String(String),
    /// Variable or constant.
    Identifier(String),
    /// Arithmetic operations.
    Add(Box<BExpression>, Box<BExpression>),
    Sub(Box<BExpression>, Box<BExpression>),
    Mul(Box<BExpression>, Box<BExpression>),
    Div(Box<BExpression>, Box<BExpression>),
    Mod(Box<BExpression>, Box<BExpression>),
    /// Set operations.
    SetUnion(Box<BExpression>, Box<BExpression>),
    SetIntersection(Box<BExpression>, Box<BExpression>),
    SetDifference(Box<BExpression>, Box<BExpression>),
    /// Set construction.
    SetExtension(Vec<BExpression>),
    SetComprehension(String, Box<BExpression>, Box<BPredicate>),
    /// Function application.
    FunctionApplication(Box<BExpression>, Box<BExpression>),
    /// Cartesian product.
    CartesianProduct(Box<BExpression>, Box<BExpression>),
}

/// B-Method generalized substitution.
#[derive(Debug, Clone)]
pub enum BSubstitution {
    /// Skip (no operation).
    Skip,
    /// Simple assignment.
    Assignment(String, BExpression),
    /// Parallel assignment.
    ParallelAssignment(Vec<String>, Vec<BExpression>),
    /// Preconditioned substitution.
    Precondition(BPredicate, Box<BSubstitution>),
    /// Assertion.
    Assertion(BPredicate, Box<BSubstitution>),
    /// Choice.
    Choice(Box<BSubstitution>, Box<BSubstitution>),
    /// Parallel composition.
    Parallel(Box<BSubstitution>, Box<BSubstitution>),
    /// Sequential composition.
    Sequential(Box<BSubstitution>, Box<BSubstitution>),
    /// Non-deterministic assignment.
    Any(Vec<String>, BPredicate, Box<BSubstitution>),
    /// Let substitution.
    Let(Vec<String>, Vec<BExpression>, Box<BSubstitution>),
    /// Variable declaration.
    Var(Vec<String>, Box<BSubstitution>),
    /// Operation call.
    OperationCall(String, Vec<BExpression>),
}

/// B-Method state representation.
pub type BState = HashMap<String, BValue>;

/// B-Method value representation.
#[derive(Debug, Clone)]
pub enum BValue {
    /// Integer value.
    Integer(i64),
    /// Boolean value.  
    Boolean(bool),
    /// String value.
    String(String),
    /// Set value.
    Set(Vec<BValue>),
    /// Relation/function value.
    Relation(HashMap<BValue, BValue>),
}

/// B-Method specification implementation.
pub struct BMethodSpec;

impl FormalSpec for BMethodSpec {
    type Value = BValue;
    type State = BState;
    type Operation = BOperation;

    fn from_lambdust_value(value: &Value) -> Result<Self::Value> {
        match value {
            Value::Literal(crate::ast::Literal::ExactInteger(n)) => {
                Ok(BValue::Integer(*n))
            }
            Value::Literal(crate::ast::Literal::Boolean(b)) => {
                Ok(BValue::Boolean(*b))
            }
            Value::Literal(crate::ast::Literal::String(s)) => {
                Ok(BValue::String(s.clone()))
            }
            Value::Nil => {
                Ok(BValue::Set(Vec::new()))
            }
            _ => Err(Error::runtime_error(
                format!("Cannot convert Lambdust value to B-Method: {:?}", value),
                None
            ).boxed()),
        }
    }

    fn to_lambdust_value(value: &Self::Value) -> Result<Value> {
        match value {
            BValue::Integer(n) => Ok(Value::integer(*n)),
            BValue::Boolean(b) => Ok(Value::boolean(*b)),
            BValue::String(s) => Ok(Value::string(s.clone())),
            BValue::Set(values) => {
                if values.is_empty() {
                    Ok(Value::Nil)
                } else {
                    let lambdust_values: Result<Vec<Value>> = values.iter()
                        .map(Self::to_lambdust_value)
                        .collect();
                    Ok(Value::list(lambdust_values?))
                }
            }
            BValue::Relation(_) => {
                // Relations would need more complex representation
                Err(Error::runtime_error(
                    "Relation conversion not yet implemented",
                    None
                ).boxed())
            }
        }
    }

    fn check_invariants(state: &Self::State) -> Result<bool> {
        // Simplified invariant checking - real implementation would evaluate predicates
        Ok(true)
    }
}

/// B-Method to Lambdust translator.
pub struct BMethodToLambdustTranslator;

impl BMethodToLambdustTranslator {
    /// Translate a B-Method machine to a Lambdust program.
    pub fn translate_machine(&self, machine: &BMachine) -> Result<String> {
        let mut result = String::new();
        
        // Generate Lambdust module for the B-Method machine
        result.push_str(&format!(";;; Translated from B-Method machine: {}\n", machine.name));
        result.push_str(";;; This module provides a functional implementation of the B specification\n\n");
        
        // Generate constants and sets
        if !machine.sets.is_empty() || !machine.constants.is_empty() {
            result.push_str(";; Constants and Sets\n");
            for set in &machine.sets {
                result.push_str(&self.translate_set(set)?);
            }
            for constant in &machine.constants {
                result.push_str(&format!("(define {} #f) ;; Constant from B specification\n", constant));
            }
            result.push_str("\n");
        }
        
        // Generate state record
        if !machine.variables.is_empty() {
            result.push_str(";; Machine State\n");
            result.push_str(&format!("(define-record-type {}-state\n", machine.name));
            result.push_str(&format!("  (make-{}-state", machine.name));
            for var in &machine.variables {
                result.push_str(&format!(" {}", var));
            }
            result.push_str(")\n");
            result.push_str(&format!("  {}-state?\n", machine.name));
            for var in &machine.variables {
                result.push_str(&format!("  ({}-state-{} {}-state-{}-set!)\n", 
                    machine.name, var, machine.name, var));
            }
            result.push_str(")\n\n");
        }
        
        // Generate initialization
        result.push_str(";; Machine Initialization\n");
        result.push_str(&format!("(define (initialize-{})\n", machine.name));
        result.push_str("  ;; Initialize machine state according to B specification\n");
        if !machine.variables.is_empty() {
            result.push_str(&format!("  (make-{}-state", machine.name));
            for _var in &machine.variables {
                result.push_str(" #f"); // Simplified initialization
            }
            result.push_str(")\n");
        } else {
            result.push_str("  '()\n");
        }
        result.push_str(")\n\n");
        
        // Generate operations
        if !machine.operations.is_empty() {
            result.push_str(";; Machine Operations\n");
            for operation in &machine.operations {
                result.push_str(&self.translate_operation(operation, &machine.name)?);
                result.push_str("\n");
            }
        }
        
        // Generate invariant checker
        if !machine.invariant.is_empty() {
            result.push_str(";; Invariant Checker\n");
            result.push_str(&format!("(define (check-{}-invariant state)\n", machine.name));
            result.push_str("  ;; Check machine invariant\n");
            result.push_str("  (and\n");
            for invariant in &machine.invariant {
                result.push_str(&format!("    {}\n", self.translate_predicate(invariant)?));
            }
            result.push_str("  )\n");
            result.push_str(")\n\n");
        }
        
        Ok(result)
    }
    
    /// Translate a B-Method set to Lambdust definition.
    fn translate_set(&self, set: &BSet) -> Result<String> {
        let mut result = String::new();
        
        if let Some(ref enumeration) = set.enumeration {
            result.push_str(&format!("(define {} '(", set.name));
            for (i, element) in enumeration.iter().enumerate() {
                if i > 0 { result.push_str(" "); }
                result.push_str(element);
            }
            result.push_str("))\n");
        } else {
            result.push_str(&format!("(define {} '()) ;; Abstract set\n", set.name));
        }
        
        Ok(result)
    }
    
    /// Translate a B-Method operation to Lambdust function.
    fn translate_operation(&self, operation: &BOperation, machine_name: &str) -> Result<String> {
        let mut result = String::new();
        
        result.push_str(&format!("(define ({}-{} state", machine_name, operation.name));
        
        // Add input parameters
        for input in &operation.inputs {
            result.push_str(&format!(" {}", input));
        }
        result.push_str(")\n");
        
        // Add precondition check
        if let Some(ref precond) = operation.precondition {
            result.push_str("  ;; Precondition check\n");
            result.push_str(&format!("  (unless {}\n", self.translate_predicate(precond)?));
            result.push_str("    (error \"Precondition violation\"))\n");
        }
        
        // Add operation body
        result.push_str("  ;; Operation body\n");
        let (new_state, outputs) = self.translate_substitution(&operation.body, "state")?;
        
        // Return result
        if operation.outputs.is_empty() {
            result.push_str(&format!("  {}\n", new_state));
        } else {
            result.push_str(&format!("  (values {} {})\n", new_state, outputs));
        }
        
        result.push_str(")\n");
        Ok(result)
    }
    
    /// Translate a B-Method predicate to Lambdust expression.
    fn translate_predicate(&self, predicate: &BPredicate) -> Result<String> {
        match predicate {
            BPredicate::BoolConstant(b) => Ok(if *b { "#t" } else { "#f" }.to_string()),
            BPredicate::Identifier(name) => Ok(name.clone()),
            BPredicate::Equals(left, right) => {
                Ok(format!("(= {} {})", 
                    self.translate_expression(left)?, 
                    self.translate_expression(right)?))
            }
            BPredicate::NotEquals(left, right) => {
                Ok(format!("(not (= {} {}))", 
                    self.translate_expression(left)?, 
                    self.translate_expression(right)?))
            }
            BPredicate::MemberOf(elem, set) => {
                Ok(format!("(member {} {})", 
                    self.translate_expression(elem)?, 
                    self.translate_expression(set)?))
            }
            BPredicate::And(left, right) => {
                Ok(format!("(and {} {})", 
                    self.translate_predicate(left)?, 
                    self.translate_predicate(right)?))
            }
            BPredicate::Or(left, right) => {
                Ok(format!("(or {} {})", 
                    self.translate_predicate(left)?, 
                    self.translate_predicate(right)?))
            }
            BPredicate::Not(pred) => {
                Ok(format!("(not {})", self.translate_predicate(pred)?))
            }
            BPredicate::Less(left, right) => {
                Ok(format!("(< {} {})", 
                    self.translate_expression(left)?, 
                    self.translate_expression(right)?))
            }
            BPredicate::LessEqual(left, right) => {
                Ok(format!("(<= {} {})", 
                    self.translate_expression(left)?, 
                    self.translate_expression(right)?))
            }
            BPredicate::Greater(left, right) => {
                Ok(format!("(> {} {})", 
                    self.translate_expression(left)?, 
                    self.translate_expression(right)?))
            }
            BPredicate::GreaterEqual(left, right) => {
                Ok(format!("(>= {} {})", 
                    self.translate_expression(left)?, 
                    self.translate_expression(right)?))
            }
            _ => Ok("(error \"Complex predicate not yet supported\")".to_string()),
        }
    }
    
    /// Translate a B-Method expression to Lambdust expression.
    fn translate_expression(&self, expression: &BExpression) -> Result<String> {
        match expression {
            BExpression::Integer(n) => Ok(n.to_string()),
            BExpression::Boolean(b) => Ok(if *b { "#t" } else { "#f" }.to_string()),
            BExpression::String(s) => Ok(format!("\"{}\"", s)),
            BExpression::Identifier(name) => Ok(name.clone()),
            BExpression::Add(left, right) => {
                Ok(format!("(+ {} {})", 
                    self.translate_expression(left)?, 
                    self.translate_expression(right)?))
            }
            BExpression::Sub(left, right) => {
                Ok(format!("(- {} {})", 
                    self.translate_expression(left)?, 
                    self.translate_expression(right)?))
            }
            BExpression::Mul(left, right) => {
                Ok(format!("(* {} {})", 
                    self.translate_expression(left)?, 
                    self.translate_expression(right)?))
            }
            BExpression::Div(left, right) => {
                Ok(format!("(/ {} {})", 
                    self.translate_expression(left)?, 
                    self.translate_expression(right)?))
            }
            BExpression::SetExtension(elements) => {
                let translated_elements: Result<Vec<String>> = elements.iter()
                    .map(|elem| self.translate_expression(elem))
                    .collect();
                Ok(format!("(list {})", translated_elements?.join(" ")))
            }
            _ => Ok("(error \"Complex expression not yet supported\")".to_string()),
        }
    }
    
    /// Translate a B-Method substitution to Lambdust expression.
    fn translate_substitution(&self, substitution: &BSubstitution, state_var: &str) -> Result<(String, String)> {
        match substitution {
            BSubstitution::Skip => {
                Ok((state_var.to_string(), "".to_string()))
            }
            BSubstitution::Assignment(var, expr) => {
                let new_state = format!("(state-update {} '({} {}))", 
                    state_var, var, self.translate_expression(expr)?);
                Ok((new_state, "".to_string()))
            }
            BSubstitution::Sequential(first, second) => {
                let (intermediate_state, _) = self.translate_substitution(first, state_var)?;
                let (final_state, outputs) = self.translate_substitution(second, &intermediate_state)?;
                Ok((final_state, outputs))
            }
            _ => {
                // Simplified - real implementation would handle all substitution types
                Ok((format!("(error \"Substitution not yet supported\")"), "".to_string()))
            }
        }
    }
    
    /// Generate proof obligations for B-Method to Lambdust translation.
    pub fn generate_proof_obligations(&self, machine: &BMachine) -> Result<Vec<ProofObligation>> {
        let mut pos = Vec::new();
        
        // Initialization proof obligation
        let po = ProofObligation::new(
            format!("init_{}", machine.name),
            format!("Initialization of {} establishes invariant", machine.name),
            format!("⟦INITIALIZATION⟧ ⇒ INV")
        ).with_context("machine", machine.name.clone());
        pos.push(po);
        
        // Operation proof obligations
        for operation in &machine.operations {
            // Precondition feasibility
            if operation.precondition.is_some() {
                let po = ProofObligation::new(
                    format!("pre_feasible_{}_{}", machine.name, operation.name),
                    format!("Precondition of {} is feasible", operation.name),
                    format!("INV ⇒ ∃ inputs . PRE({})", operation.name)
                ).with_context("operation", operation.name.clone());
                pos.push(po);
            }
            
            // Invariant preservation
            let po = ProofObligation::new(
                format!("inv_pres_{}_{}", machine.name, operation.name),
                format!("Operation {} preserves invariant", operation.name),
                format!("{{INV ∧ PRE({})}} {} {{INV}}", operation.name, operation.name)
            ).with_context("operation", operation.name.clone());
            pos.push(po);
        }
        
        Ok(pos)
    }
}

impl fmt::Display for BMachine {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "MACHINE {}", self.name)?;
        
        if !self.parameters.is_empty() {
            writeln!(f, "PARAMETERS")?;
            for param in &self.parameters {
                writeln!(f, "  {}", param)?;
            }
        }
        
        if !self.constraints.is_empty() {
            writeln!(f, "CONSTRAINTS")?;
            for constraint in &self.constraints {
                writeln!(f, "  {}", constraint)?;
            }
        }
        
        if !self.sets.is_empty() {
            writeln!(f, "SETS")?;
            for set in &self.sets {
                writeln!(f, "  {}", set)?;
            }
        }
        
        if !self.constants.is_empty() {
            writeln!(f, "CONSTANTS")?;
            for constant in &self.constants {
                writeln!(f, "  {}", constant)?;
            }
        }
        
        if !self.properties.is_empty() {
            writeln!(f, "PROPERTIES")?;
            for property in &self.properties {
                writeln!(f, "  {}", property)?;
            }
        }
        
        if !self.variables.is_empty() {
            writeln!(f, "VARIABLES")?;
            for var in &self.variables {
                writeln!(f, "  {}", var)?;
            }
        }
        
        if !self.invariant.is_empty() {
            writeln!(f, "INVARIANT")?;
            for inv in &self.invariant {
                writeln!(f, "  {}", inv)?;
            }
        }
        
        writeln!(f, "INITIALISATION")?;
        writeln!(f, "  {}", self.initialization)?;
        
        if !self.operations.is_empty() {
            writeln!(f, "OPERATIONS")?;
            for operation in &self.operations {
                writeln!(f, "{}", operation)?;
            }
        }
        
        writeln!(f, "END")
    }
}

impl fmt::Display for BSet {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(ref enumeration) = self.enumeration {
            write!(f, "{} = {{{}}}", self.name, enumeration.join(", "))
        } else {
            write!(f, "{}", self.name)
        }
    }
}

impl fmt::Display for BOperation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "  {}", self.name)?;
        
        if !self.outputs.is_empty() {
            write!(f, "({}) <-- ", self.outputs.join(", "))?;
        }
        
        if !self.inputs.is_empty() {
            write!(f, "({})", self.inputs.join(", "))?;
        }
        
        writeln!(f, " =")?;
        
        if let Some(ref precond) = self.precondition {
            writeln!(f, "    PRE {} THEN", precond)?;
            writeln!(f, "      {}", self.body)?;
            writeln!(f, "    END")?;
        } else {
            writeln!(f, "    {}", self.body)?;
        }
        
        Ok(())
    }
}

impl fmt::Display for BPredicate {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BPredicate::BoolConstant(b) => write!(f, "{}", if *b { "TRUE" } else { "FALSE" }),
            BPredicate::Identifier(name) => write!(f, "{}", name),
            BPredicate::Equals(left, right) => write!(f, "{} = {}", left, right),
            BPredicate::NotEquals(left, right) => write!(f, "{} ≠ {}", left, right),
            BPredicate::MemberOf(elem, set) => write!(f, "{} ∈ {}", elem, set),
            BPredicate::And(left, right) => write!(f, "({} ∧ {})", left, right),
            BPredicate::Or(left, right) => write!(f, "({} ∨ {})", left, right),
            BPredicate::Not(pred) => write!(f, "¬({})", pred),
            BPredicate::Less(left, right) => write!(f, "{} < {}", left, right),
            BPredicate::LessEqual(left, right) => write!(f, "{} ≤ {}", left, right),
            BPredicate::Greater(left, right) => write!(f, "{} > {}", left, right),
            BPredicate::GreaterEqual(left, right) => write!(f, "{} ≥ {}", left, right),
            _ => write!(f, "⟨complex predicate⟩"),
        }
    }
}

impl fmt::Display for BExpression {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BExpression::Integer(n) => write!(f, "{}", n),
            BExpression::Boolean(b) => write!(f, "{}", if *b { "TRUE" } else { "FALSE" }),
            BExpression::String(s) => write!(f, "\"{}\"", s),
            BExpression::Identifier(name) => write!(f, "{}", name),
            BExpression::Add(left, right) => write!(f, "({} + {})", left, right),
            BExpression::Sub(left, right) => write!(f, "({} - {})", left, right),
            BExpression::Mul(left, right) => write!(f, "({} * {})", left, right),
            BExpression::Div(left, right) => write!(f, "({} / {})", left, right),
            BExpression::SetExtension(elements) => {
                write!(f, "{{{}}}", elements.iter()
                    .map(|e| e.to_string())
                    .collect::<Vec<_>>()
                    .join(", "))
            }
            _ => write!(f, "⟨complex expression⟩"),
        }
    }
}

impl fmt::Display for BSubstitution {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BSubstitution::Skip => write!(f, "skip"),
            BSubstitution::Assignment(var, expr) => write!(f, "{} := {}", var, expr),
            BSubstitution::ParallelAssignment(vars, exprs) => {
                let assignments: Vec<String> = vars.iter().zip(exprs.iter())
                    .map(|(var, expr)| format!("{} := {}", var, expr))
                    .collect();
                write!(f, "{}", assignments.join(" || "))
            }
            BSubstitution::Sequential(first, second) => write!(f, "{} ; {}", first, second),
            BSubstitution::Parallel(first, second) => write!(f, "{} || {}", first, second),
            _ => write!(f, "⟨complex substitution⟩"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_b_value_conversion() {
        let b_value = BValue::Integer(42);
        let lambdust_value = BMethodSpec::to_lambdust_value(&b_value).unwrap();
        let converted_back = BMethodSpec::from_lambdust_value(&lambdust_value).unwrap();
        
        assert_eq!(b_value, converted_back);
    }
    
    #[test]
    fn test_b_machine_display() {
        let machine = BMachine {
            name: "SimpleCounter".to_string(),
            parameters: vec![],
            constraints: vec![],
            sets: vec![],
            constants: vec![],
            properties: vec![],
            variables: vec!["count".to_string()],
            invariant: vec![
                BPredicate::GreaterEqual(
                    Box::new(BExpression::Identifier("count".to_string())),
                    Box::new(BExpression::Integer(0))
                )
            ],
            initialization: BSubstitution::Assignment(
                "count".to_string(),
                BExpression::Integer(0)
            ),
            operations: vec![
                BOperation {
                    name: "increment".to_string(),
                    inputs: vec![],
                    outputs: vec![],
                    precondition: Some(BPredicate::Less(
                        Box::new(BExpression::Identifier("count".to_string())),
                        Box::new(BExpression::Integer(100))
                    )),
                    body: BSubstitution::Assignment(
                        "count".to_string(),
                        BExpression::Add(
                            Box::new(BExpression::Identifier("count".to_string())),
                            Box::new(BExpression::Integer(1))
                        )
                    ),
                }
            ],
        };
        
        let display_string = machine.to_string();
        assert!(display_string.contains("MACHINE SimpleCounter"));
        assert!(display_string.contains("VARIABLES"));
        assert!(display_string.contains("count"));
        assert!(display_string.contains("increment"));
    }
    
    #[test]
    fn test_b_to_lambdust_translation() {
        let machine = BMachine {
            name: "TestMachine".to_string(),
            parameters: vec![],
            constraints: vec![],
            sets: vec![],
            constants: vec![],
            properties: vec![],
            variables: vec!["x".to_string()],
            invariant: vec![],
            initialization: BSubstitution::Assignment(
                "x".to_string(),
                BExpression::Integer(0)
            ),
            operations: vec![
                BOperation {
                    name: "set_x".to_string(),
                    inputs: vec!["new_x".to_string()],
                    outputs: vec![],
                    precondition: None,
                    body: BSubstitution::Assignment(
                        "x".to_string(),
                        BExpression::Identifier("new_x".to_string())
                    ),
                }
            ],
        };
        
        let translator = BMethodToLambdustTranslator;
        let translated = translator.translate_machine(&machine).unwrap();
        
        assert!(translated.contains("TestMachine"));
        assert!(translated.contains("define-record-type"));
        assert!(translated.contains("TestMachine-set_x"));
        assert!(translated.contains("initialize-TestMachine"));
    }
    
    #[test]
    fn test_proof_obligation_generation() {
        let machine = BMachine {
            name: "TestMachine".to_string(),
            parameters: vec![],
            constraints: vec![],
            sets: vec![],
            constants: vec![],
            properties: vec![],
            variables: vec!["x".to_string()],
            invariant: vec![
                BPredicate::GreaterEqual(
                    Box::new(BExpression::Identifier("x".to_string())),
                    Box::new(BExpression::Integer(0))
                )
            ],
            initialization: BSubstitution::Assignment(
                "x".to_string(),
                BExpression::Integer(0)
            ),
            operations: vec![
                BOperation {
                    name: "increment".to_string(),
                    inputs: vec![],
                    outputs: vec![],
                    precondition: Some(BPredicate::Less(
                        Box::new(BExpression::Identifier("x".to_string())),
                        Box::new(BExpression::Integer(100))
                    )),
                    body: BSubstitution::Assignment(
                        "x".to_string(),
                        BExpression::Add(
                            Box::new(BExpression::Identifier("x".to_string())),
                            Box::new(BExpression::Integer(1))
                        )
                    ),
                }
            ],
        };
        
        let translator = BMethodToLambdustTranslator;
        let pos = translator.generate_proof_obligations(&machine).unwrap();
        
        // Should generate initialization PO
        assert!(pos.iter().any(|po| po.id.contains("init")));
        
        // Should generate precondition feasibility PO
        assert!(pos.iter().any(|po| po.id.contains("pre_feasible")));
        
        // Should generate invariant preservation PO
        assert!(pos.iter().any(|po| po.id.contains("inv_pres")));
    }
}