//! Event-B integration for Lambdust.
//!
//! This module provides tools for translating between Event-B machines and
//! Lambdust programs, with formal correctness guarantees.

use super::{FormalSpec, Translation, ProofObligation, TranslationCertificate};
use crate::eval::Value;
use crate::diagnostics::{Error, Result, Span};
use std::collections::HashMap;
use std::fmt;

/// Represents an Event-B machine.
#[derive(Debug, Clone)]
pub struct EventBMachine {
    /// Machine name.
    pub name: String,
    /// Machine variables.
    pub variables: Vec<String>,
    /// Machine invariants.
    pub invariants: Vec<EventBPredicate>,
    /// Machine initialization.
    pub initialization: EventBAction,
    /// Machine events.
    pub events: Vec<EventBEvent>,
}

/// Represents an Event-B event.
#[derive(Debug, Clone)]
pub struct EventBEvent {
    /// Event name.
    pub name: String,
    /// Event parameters.
    pub parameters: Vec<String>,
    /// Event guards.
    pub guards: Vec<EventBPredicate>,
    /// Event actions.
    pub actions: Vec<EventBAction>,
    /// Event status (ordinary, convergent, anticipated).
    pub status: EventStatus,
}

/// Event-B event status.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EventStatus {
    /// Ordinary event.
    Ordinary,
    /// Convergent event (must decrease variant).
    Convergent,
    /// Anticipated event (may increase variant).
    Anticipated,
}

/// Represents an Event-B predicate.
#[derive(Debug, Clone)]
pub enum EventBPredicate {
    /// Boolean constant.
    BoolConstant(bool),
    /// Variable reference.
    Variable(String),
    /// Equality predicate.
    Equals(Box<EventBExpression>, Box<EventBExpression>),
    /// Set membership.
    In(Box<EventBExpression>, Box<EventBExpression>),
    /// Logical conjunction.
    And(Box<EventBPredicate>, Box<EventBPredicate>),
    /// Logical disjunction.
    Or(Box<EventBPredicate>, Box<EventBPredicate>),
    /// Logical negation.
    Not(Box<EventBPredicate>),
    /// Universal quantification.
    Forall(Vec<String>, Box<EventBPredicate>),
    /// Existential quantification.
    Exists(Vec<String>, Box<EventBPredicate>),
}

/// Represents an Event-B expression.
#[derive(Debug, Clone)]
pub enum EventBExpression {
    /// Integer literal.
    Integer(i64),
    /// Boolean literal.
    Boolean(bool),
    /// Variable reference.
    Variable(String),
    /// Arithmetic addition.
    Add(Box<EventBExpression>, Box<EventBExpression>),
    /// Arithmetic subtraction.
    Sub(Box<EventBExpression>, Box<EventBExpression>),
    /// Set construction.
    SetConstruction(Vec<EventBExpression>),
    /// Function application.
    FunctionApplication(String, Vec<EventBExpression>),
}

/// Represents an Event-B action (generalized substitution).
#[derive(Debug, Clone)]
pub enum EventBAction {
    /// Skip action (no-op).
    Skip,
    /// Simple assignment.
    Assignment(String, EventBExpression),
    /// Non-deterministic assignment from a set.
    AssignmentFromSet(String, EventBExpression),
    /// Parallel composition.
    Parallel(Vec<EventBAction>),
    /// Sequential composition.
    Sequential(Vec<EventBAction>),
    /// Choice composition.
    Choice(Vec<EventBAction>),
    /// Guarded action.
    Guarded(EventBPredicate, Box<EventBAction>),
}

/// Event-B state representation.
pub type EventBState = HashMap<String, EventBValue>;

/// Event-B value representation.
#[derive(Debug, Clone)]
pub enum EventBValue {
    /// Integer value.
    Integer(i64),
    /// Boolean value.
    Boolean(bool),
    /// Set value.
    Set(Vec<EventBValue>),
    /// Function value (represented as a map).
    Function(HashMap<EventBValue, EventBValue>),
}

/// Event-B specification implementation.
pub struct EventBSpec;

impl FormalSpec for EventBSpec {
    type Value = EventBValue;
    type State = EventBState;
    type Operation = EventBEvent;

    fn from_lambdust_value(value: &Value) -> Result<Self::Value> {
        match value {
            Value::Literal(crate::ast::Literal::ExactInteger(n)) => {
                Ok(EventBValue::Integer(*n))
            }
            Value::Literal(crate::ast::Literal::Boolean(b)) => {
                Ok(EventBValue::Boolean(*b))
            }
            Value::Nil => {
                Ok(EventBValue::Set(Vec::new()))
            }
            _ => Err(Box::new(Error::runtime_error(
                format!("Cannot convert Lambdust value to Event-B: {value:?}"),
                None
            ).boxed()))
        }
    }

    fn to_lambdust_value(value: &Self::Value) -> Result<Value> {
        match value {
            EventBValue::Integer(n) => {
                Ok(Value::integer(*n))
            }
            EventBValue::Boolean(b) => {
                Ok(Value::boolean(*b))
            }
            EventBValue::Set(values) => {
                if values.is_empty() {
                    Ok(Value::Nil)
                } else {
                    // Convert to list for now
                    let lambdust_values: Result<Vec<Value>> = values.iter()
                        .map(Self::to_lambdust_value)
                        .collect();
                    Ok(Value::list(lambdust_values?))
                }
            }
            EventBValue::Function(_) => {
                // Functions would need more complex representation
                Err(Box::new(Error::runtime_error(
                    "Function conversion not yet implemented".to_string(),
                    None
                ).boxed()))
            }
        }
    }

    fn check_invariants(state: &Self::State) -> Result<bool> {
        // Simplified invariant checking - real implementation would evaluate predicates
        Ok(true)
    }
}

/// Event-B to Lambdust translator.
pub struct EventBToLambdustTranslator;

impl EventBToLambdustTranslator {
    /// Translate an Event-B machine to a Lambdust program.
    pub fn translate_machine(&self, machine: &EventBMachine) -> Result<String> {
        let mut result = String::new();
        
        // Generate Lambdust actor for the Event-B machine
        result.push_str(&format!(";;; Translated from Event-B machine: {}\n", machine.name));
        result.push_str(&format!("(define {}-actor\n", machine.name));
        result.push_str("  (spawn\n");
        result.push_str("    (lambda ()\n");
        
        // Initialize variables
        result.push_str("      ;; Variable initialization\n");
        for var in &machine.variables {
            result.push_str(&format!("      (define {var} #f) ;; Initialize {var}\n"));
        }
        
        // Event handling loop
        result.push_str("      ;; Event handling\n");
        result.push_str("      (let loop ()\n");
        result.push_str("        (receive\n");
        
        for event in &machine.events {
            result.push_str(&self.translate_event(event)?);
        }
        
        result.push_str("        )\n");
        result.push_str("        (loop)\n");
        result.push_str("      )\n");
        result.push_str("    )\n");
        result.push_str("  )\n");
        result.push_str(")\n");
        
        Ok(result)
    }
    
    /// Translate an Event-B event to Lambdust pattern matching.
    fn translate_event(&self, event: &EventBEvent) -> Result<String> {
        let mut result = String::new();
        
        result.push_str(&format!("          ;; Event: {}\n", event.name));
        result.push_str(&format!("          (({}", event.name));
        
        // Add parameters
        for param in &event.parameters {
            result.push_str(&format!(" {param}"));
        }
        result.push_str(")\n");
        
        // Add guards as conditions
        if !event.guards.is_empty() {
            result.push_str("            (when (and\n");
            for guard in &event.guards {
                result.push_str(&format!("              {}\n", self.translate_predicate(guard)?));
            }
            result.push_str("            )\n");
        }
        
        // Add actions
        result.push_str("              ;; Actions\n");
        for action in &event.actions {
            result.push_str(&format!("              {}\n", self.translate_action(action)?));
        }
        
        if !event.guards.is_empty() {
            result.push_str("            )\n");
        }
        result.push_str("          )\n");
        
        Ok(result)
    }
    
    /// Translate an Event-B predicate to Lambdust expression.
    fn translate_predicate(&self, predicate: &EventBPredicate) -> Result<String> {
        match predicate {
            EventBPredicate::BoolConstant(b) => Ok(if *b { "#t" } else { "#f" }.to_string()),
            EventBPredicate::Variable(var) => Ok(var.clone()),
            EventBPredicate::Equals(left, right) => {
                Ok(format!("(= {} {})", 
                    self.translate_expression(left)?, 
                    self.translate_expression(right)?))
            }
            EventBPredicate::In(elem, set) => {
                Ok(format!("(member {} {})", 
                    self.translate_expression(elem)?, 
                    self.translate_expression(set)?))
            }
            EventBPredicate::And(left, right) => {
                Ok(format!("(and {} {})", 
                    self.translate_predicate(left)?, 
                    self.translate_predicate(right)?))
            }
            EventBPredicate::Or(left, right) => {
                Ok(format!("(or {} {})", 
                    self.translate_predicate(left)?, 
                    self.translate_predicate(right)?))
            }
            EventBPredicate::Not(pred) => {
                Ok(format!("(not {})", self.translate_predicate(pred)?))
            }
            EventBPredicate::Forall(vars, pred) => {
                // Simplified - real implementation would be more complex
                Ok(format!("(forall ({}) {})", 
                    vars.join(" "), 
                    self.translate_predicate(pred)?))
            }
            EventBPredicate::Exists(vars, pred) => {
                Ok(format!("(exists ({}) {})", 
                    vars.join(" "), 
                    self.translate_predicate(pred)?))
            }
        }
    }
    
    /// Translate an Event-B expression to Lambdust expression.
    fn translate_expression(&self, expression: &EventBExpression) -> Result<String> {
        match expression {
            EventBExpression::Integer(n) => Ok(n.to_string()),
            EventBExpression::Boolean(b) => Ok(if *b { "#t" } else { "#f" }.to_string()),
            EventBExpression::Variable(var) => Ok(var.clone()),
            EventBExpression::Add(left, right) => {
                Ok(format!("(+ {} {})", 
                    self.translate_expression(left)?, 
                    self.translate_expression(right)?))
            }
            EventBExpression::Sub(left, right) => {
                Ok(format!("(- {} {})", 
                    self.translate_expression(left)?, 
                    self.translate_expression(right)?))
            }
            EventBExpression::SetConstruction(elements) => {
                let translated_elements: Result<Vec<String>> = elements.iter()
                    .map(|elem| self.translate_expression(elem))
                    .collect();
                Ok(format!("(list {})", translated_elements?.join(" ")))
            }
            EventBExpression::FunctionApplication(name, args) => {
                let translated_args: Result<Vec<String>> = args.iter()
                    .map(|arg| self.translate_expression(arg))
                    .collect();
                Ok(format!("({} {})", name, translated_args?.join(" ")))
            }
        }
    }
    
    /// Translate an Event-B action to Lambdust expression.
    fn translate_action(&self, action: &EventBAction) -> Result<String> {
        match action {
            EventBAction::Skip => Ok("(void)".to_string()),
            EventBAction::Assignment(var, expr) => {
                Ok(format!("(set! {} {})", var, self.translate_expression(expr)?))
            }
            EventBAction::AssignmentFromSet(var, set) => {
                Ok(format!("(set! {} (choose-from {}))", 
                    var, self.translate_expression(set)?))
            }
            EventBAction::Parallel(actions) => {
                let translated_actions: Result<Vec<String>> = actions.iter()
                    .map(|action| self.translate_action(action))
                    .collect();
                Ok(format!("(begin {})", translated_actions?.join(" ")))
            }
            EventBAction::Sequential(actions) => {
                let translated_actions: Result<Vec<String>> = actions.iter()
                    .map(|action| self.translate_action(action))
                    .collect();
                Ok(format!("(begin {})", translated_actions?.join(" ")))
            }
            EventBAction::Choice(actions) => {
                // Non-deterministic choice - simplified implementation
                let translated_actions: Result<Vec<String>> = actions.iter()
                    .map(|action| self.translate_action(action))
                    .collect();
                Ok(format!("(choice {})", translated_actions?.join(" ")))
            }
            EventBAction::Guarded(guard, action) => {
                Ok(format!("(when {} {})", 
                    self.translate_predicate(guard)?, 
                    self.translate_action(action)?))
            }
        }
    }
    
    /// Generate proof obligations for Event-B to Lambdust translation.
    pub fn generate_proof_obligations(
        &self, 
        machine: &EventBMachine
    ) -> Result<Vec<ProofObligation>> {
        let mut pos = Vec::new();
        
        // Invariant preservation proof obligations
        for (i, invariant) in machine.invariants.iter().enumerate() {
            for event in &machine.events {
                let po = ProofObligation::new(
                    format!("inv_preservation_{}_{}", i, event.name),
                    format!("Event {} preserves invariant {}", event.name, i),
                    format!("{{INV({})}} {} {{INV({})}}", i, event.name, i)
                ).with_context("machine", machine.name.clone())
                 .with_context("event", event.name.clone());
                pos.push(po);
            }
        }
        
        // Guard feasibility proof obligations
        for event in &machine.events {
            if !event.guards.is_empty() {
                let po = ProofObligation::new(
                    format!("guard_feasible_{}", event.name),
                    format!("Guards of event {} are feasible", event.name),
                    format!("feasible(guards({}))", event.name)
                ).with_context("event", event.name.clone());
                pos.push(po);
            }
        }
        
        // Action well-definedness proof obligations
        for event in &machine.events {
            for (i, action) in event.actions.iter().enumerate() {
                let po = ProofObligation::new(
                    format!("action_wd_{}_{}_{}", event.name, i, "action"),
                    format!("Action {} of event {} is well-defined", i, event.name),
                    format!("well_defined(action_{}({}))", i, event.name)
                ).with_context("event", event.name.clone())
                 .with_context("action_index", i.to_string());
                pos.push(po);
            }
        }
        
        Ok(pos)
    }
}

impl fmt::Display for EventBMachine {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "MACHINE {}", self.name)?;
        
        if !self.variables.is_empty() {
            writeln!(f, "VARIABLES")?;
            for var in &self.variables {
                writeln!(f, "  {var}")?;
            }
        }
        
        if !self.invariants.is_empty() {
            writeln!(f, "INVARIANTS")?;
            for (i, inv) in self.invariants.iter().enumerate() {
                writeln!(f, "  inv{i}: {inv}")?;
            }
        }
        
        writeln!(f, "INITIALISATION")?;
        writeln!(f, "  {}", self.initialization)?;
        
        if !self.events.is_empty() {
            writeln!(f, "EVENTS")?;
            for event in &self.events {
                writeln!(f, "{event}")?;
            }
        }
        
        writeln!(f, "END")
    }
}

impl fmt::Display for EventBEvent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "  EVENT {}", self.name)?;
        
        if !self.parameters.is_empty() {
            writeln!(f)?;
            write!(f, "  ANY {}", self.parameters.join(", "))?;
        }
        
        if !self.guards.is_empty() {
            writeln!(f)?;
            writeln!(f, "  WHERE")?;
            for guard in &self.guards {
                writeln!(f, "    {guard}")?;
            }
        }
        
        if !self.actions.is_empty() {
            writeln!(f)?;
            writeln!(f, "  THEN")?;
            for action in &self.actions {
                writeln!(f, "    {action}")?;
            }
        }
        
        writeln!(f)?;
        writeln!(f, "  END")
    }
}

impl fmt::Display for EventBPredicate {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EventBPredicate::BoolConstant(b) => write!(f, "{}", if *b { "TRUE" } else { "FALSE" }),
            EventBPredicate::Variable(var) => write!(f, "{var}"),
            EventBPredicate::Equals(left, right) => write!(f, "{left} = {right}"),
            EventBPredicate::In(elem, set) => write!(f, "{elem} ∈ {set}"),
            EventBPredicate::And(left, right) => write!(f, "({left} ∧ {right})"),
            EventBPredicate::Or(left, right) => write!(f, "({left} ∨ {right})"),
            EventBPredicate::Not(pred) => write!(f, "¬({pred})"),
            EventBPredicate::Forall(vars, pred) => write!(f, "∀{}·({})", vars.join(","), pred),
            EventBPredicate::Exists(vars, pred) => write!(f, "∃{}·({})", vars.join(","), pred),
        }
    }
}

impl fmt::Display for EventBExpression {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EventBExpression::Integer(n) => write!(f, "{n}"),
            EventBExpression::Boolean(b) => write!(f, "{}", if *b { "TRUE" } else { "FALSE" }),
            EventBExpression::Variable(var) => write!(f, "{var}"),
            EventBExpression::Add(left, right) => write!(f, "({left} + {right})"),
            EventBExpression::Sub(left, right) => write!(f, "({left} - {right})"),
            EventBExpression::SetConstruction(elements) => {
                write!(f, "{{{}}}", elements.iter()
                    .map(|e| e.to_string())
                    .collect::<Vec<_>>()
                    .join(", "))
            }
            EventBExpression::FunctionApplication(name, args) => {
                write!(f, "{}({})", name, args.iter()
                    .map(|e| e.to_string())
                    .collect::<Vec<_>>()
                    .join(", "))
            }
        }
    }
}

impl fmt::Display for EventBAction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EventBAction::Skip => write!(f, "skip"),
            EventBAction::Assignment(var, expr) => write!(f, "{var} := {expr}"),
            EventBAction::AssignmentFromSet(var, set) => write!(f, "{var} :∈ {set}"),
            EventBAction::Parallel(actions) => {
                write!(f, "{}", actions.iter()
                    .map(|a| a.to_string())
                    .collect::<Vec<_>>()
                    .join(" ∥ "))
            }
            EventBAction::Sequential(actions) => {
                write!(f, "{}", actions.iter()
                    .map(|a| a.to_string())
                    .collect::<Vec<_>>()
                    .join("; "))
            }
            EventBAction::Choice(actions) => {
                write!(f, "{}", actions.iter()
                    .map(|a| a.to_string())
                    .collect::<Vec<_>>()
                    .join(" ⫶ "))
            }
            EventBAction::Guarded(guard, action) => write!(f, "{guard} | {action}"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_eventb_value_conversion() {
        let eb_value = EventBValue::Integer(42);
        let lambdust_value = EventBSpec::to_lambdust_value(&eb_value).unwrap();
        let converted_back = EventBSpec::from_lambdust_value(&lambdust_value).unwrap();
        
        assert_eq!(eb_value, converted_back);
    }
    
    #[test]
    fn test_eventb_machine_display() {
        let machine = EventBMachine {
            name: "TestMachine".to_string(),
            variables: vec!["x".to_string(), "y".to_string()],
            invariants: vec![
                EventBPredicate::In(
                    Box::new(EventBExpression::Variable("x".to_string())),
                    Box::new(EventBExpression::SetConstruction(vec![
                        EventBExpression::Integer(1),
                        EventBExpression::Integer(2),
                        EventBExpression::Integer(3),
                    ]))
                )
            ],
            initialization: EventBAction::Assignment(
                "x".to_string(),
                EventBExpression::Integer(1)
            ),
            events: vec![
                EventBEvent {
                    name: "increment".to_string(),
                    parameters: vec![],
                    guards: vec![
                        EventBPredicate::Equals(
                            Box::new(EventBExpression::Variable("x".to_string())),
                            Box::new(EventBExpression::Integer(1))
                        )
                    ],
                    actions: vec![
                        EventBAction::Assignment(
                            "x".to_string(),
                            EventBExpression::Add(
                                Box::new(EventBExpression::Variable("x".to_string())),
                                Box::new(EventBExpression::Integer(1))
                            )
                        )
                    ],
                    status: EventStatus::Ordinary,
                }
            ],
        };
        
        let display_string = machine.to_string();
        assert!(display_string.contains("MACHINE TestMachine"));
        assert!(display_string.contains("VARIABLES"));
        assert!(display_string.contains("EVENT increment"));
    }
    
    #[test]
    fn test_eventb_to_lambdust_translation() {
        let machine = EventBMachine {
            name: "SimpleCounter".to_string(),
            variables: vec!["count".to_string()],
            invariants: vec![],
            initialization: EventBAction::Assignment(
                "count".to_string(),
                EventBExpression::Integer(0)
            ),
            events: vec![
                EventBEvent {
                    name: "increment".to_string(),
                    parameters: vec![],
                    guards: vec![],
                    actions: vec![
                        EventBAction::Assignment(
                            "count".to_string(),
                            EventBExpression::Add(
                                Box::new(EventBExpression::Variable("count".to_string())),
                                Box::new(EventBExpression::Integer(1))
                            )
                        )
                    ],
                    status: EventStatus::Ordinary,
                }
            ],
        };
        
        let translator = EventBToLambdustTranslator;
        let translated = translator.translate_machine(&machine).unwrap();
        
        assert!(translated.contains("SimpleCounter-actor"));
        assert!(translated.contains("(define count #f)"));
        assert!(translated.contains("(increment"));
        assert!(translated.contains("(set! count (+ count 1))"));
    }
    
    #[test]
    fn test_proof_obligation_generation() {
        let machine = EventBMachine {
            name: "TestMachine".to_string(),
            variables: vec!["x".to_string()],
            invariants: vec![
                EventBPredicate::In(
                    Box::new(EventBExpression::Variable("x".to_string())),
                    Box::new(EventBExpression::SetConstruction(vec![
                        EventBExpression::Integer(1),
                        EventBExpression::Integer(2),
                    ]))
                )
            ],
            initialization: EventBAction::Assignment(
                "x".to_string(),
                EventBExpression::Integer(1)
            ),
            events: vec![
                EventBEvent {
                    name: "change".to_string(),
                    parameters: vec![],
                    guards: vec![],
                    actions: vec![
                        EventBAction::Assignment(
                            "x".to_string(),
                            EventBExpression::Integer(2)
                        )
                    ],
                    status: EventStatus::Ordinary,
                }
            ],
        };
        
        let translator = EventBToLambdustTranslator;
        let pos = translator.generate_proof_obligations(&machine).unwrap();
        
        // Should generate invariant preservation PO
        assert!(pos.iter().any(|po| po.id.contains("inv_preservation")));
        
        // Should generate action well-definedness PO  
        assert!(pos.iter().any(|po| po.id.contains("action_wd")));
    }
}