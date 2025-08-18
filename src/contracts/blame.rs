//! Blame tracking system for precise contract error attribution.
//!
//! This module implements a sophisticated blame tracking system that enables
//! precise error attribution in contract violations. The system tracks:
//!
//! - Contract boundaries and participants
//! - Call stack and evaluation context
//! - Positive and negative blame assignment
//! - Contract violation history and patterns

use crate::diagnostics::{Span, Spanned};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use std::fmt;
use std::sync::{Arc, Mutex};

/// Blame information for contract violations.
///
/// Blame tracks which party is responsible for a contract violation:
/// - Positive blame: The party that provided the value (caller/producer)
/// - Negative blame: The party that required the contract (callee/consumer)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BlameInfo {
    /// Positive blame target (who provided the value)
    pub positive: BlameTarget,
    /// Negative blame target (who required the contract)
    pub negative: BlameTarget,
    /// Contract boundary where blame was established
    pub boundary: BlameBoundary,
    /// Call stack at the time of contract establishment
    pub call_stack: Vec<CallFrame>,
    /// Unique identifier for this blame context
    pub id: BlameId,
    /// Parent blame context (for nested contracts)
    pub parent: Option<BlameId>,
}

/// Unique identifier for blame contexts.
pub type BlameId = u64;

/// Blame target identifying a responsible party.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum BlameTarget {
    /// Source location (file, line, column)
    Location {
        /// Source file name
        file: String,
        /// Location span in the source file
        span: Span,
        /// Human-readable description of the location
        description: String,
    },
    /// Function or procedure name
    Function {
        /// Name of the function
        name: String,
        /// Optional module containing the function
        module: Option<String>,
        /// Location span of the function definition
        span: Span,
    },
    /// Module boundary
    Module {
        /// Name of the module
        name: String,
        /// Interface or contract specification
        interface: String,
    },
    /// Interactive REPL input
    Repl {
        /// Sequential number of the REPL input
        input_number: usize,
        /// Location span within the input
        span: Span,
    },
    /// System/built-in functions
    System {
        /// Name of the system component
        component: String,
        /// Description of the system function
        description: String,
    },
    /// Unknown source (fallback)
    Unknown {
        /// Description of the unknown source
        description: String,
    },
}

/// Contract boundary information.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BlameBoundary {
    /// Type of boundary
    pub boundary_type: BoundaryType,
    /// Contract expression that established the boundary
    pub contract: String,
    /// Location where the boundary was established
    pub location: Span,
    /// Additional context information
    pub context: HashMap<String, String>,
}

/// Types of contract boundaries.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum BoundaryType {
    /// Function definition with contract
    FunctionDefinition,
    /// Module interface
    ModuleInterface,
    /// Explicit contract application
    ExplicitContract,
    /// Higher-order function argument
    HigherOrderArgument,
    /// Higher-order function result
    HigherOrderResult,
    /// Recursive contract unfolding
    RecursiveUnfolding,
    /// Parametric contract instantiation
    ParametricInstantiation,
}

/// Call frame for blame stack traces.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CallFrame {
    /// Function or procedure name
    pub name: String,
    /// Source location
    pub location: Span,
    /// Module name
    pub module: Option<String>,
    /// Frame type
    pub frame_type: FrameType,
}

/// Types of call frames.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum FrameType {
    /// Regular function call
    FunctionCall,
    /// Macro expansion
    MacroExpansion,
    /// Continuation invocation
    Continuation,
    /// Primitive operation
    Primitive,
    /// Contract checking
    ContractCheck,
}

/// Blame tracker for managing blame contexts and violations.
#[derive(Debug, Clone)]
pub struct BlameTracker {
    /// Counter for generating unique blame IDs
    next_id: Arc<Mutex<BlameId>>,
    /// Active blame contexts
    contexts: Arc<Mutex<HashMap<BlameId, BlameInfo>>>,
    /// Violation history
    violations: Arc<Mutex<VecDeque<BlameViolation>>>,
    /// Maximum violations to keep in history
    max_history: usize,
    /// Blame stack for nested contexts
    blame_stack: Arc<Mutex<Vec<BlameId>>>,
}

/// Record of a contract violation with blame information.
#[derive(Debug, Clone)]
pub struct BlameViolation {
    /// Blame information
    pub blame: BlameInfo,
    /// Contract that was violated
    pub contract: String,
    /// Expected value description
    pub expected: String,
    /// Actual value description
    pub actual: String,
    /// Error message
    pub message: String,
    /// Timestamp of violation
    pub timestamp: std::time::SystemTime,
    /// Violation location
    pub location: Span,
}

impl BlameTracker {
    /// Creates a new blame tracker.
    pub fn new() -> Self {
        Self {
            next_id: Arc::new(Mutex::new(1)),
            contexts: Arc::new(Mutex::new(HashMap::new())),
            violations: Arc::new(Mutex::new(VecDeque::new())),
            max_history: 1000,
            blame_stack: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// Creates a new blame tracker with custom history size.
    pub fn with_history_size(max_history: usize) -> Self {
        Self {
            next_id: Arc::new(Mutex::new(1)),
            contexts: Arc::new(Mutex::new(HashMap::new())),
            violations: Arc::new(Mutex::new(VecDeque::new())),
            max_history,
            blame_stack: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// Generates a new unique blame ID.
    pub fn new_blame_id(&self) -> BlameId {
        let mut counter = self.next_id.lock().unwrap();
        let id = *counter;
        *counter += 1;
        id
    }

    /// Creates a new blame context.
    pub fn create_blame_context(
        &self,
        positive: BlameTarget,
        negative: BlameTarget,
        boundary: BlameBoundary,
        call_stack: Vec<CallFrame>,
    ) -> BlameInfo {
        let id = self.new_blame_id();
        let parent = self.current_blame_context();
        
        let blame = BlameInfo {
            positive,
            negative,
            boundary,
            call_stack,
            id,
            parent,
        };

        // Store the context
        {
            let mut contexts = self.contexts.lock().unwrap();
            contexts.insert(id, blame.clone());
        }

        blame
    }

    /// Gets the current blame context ID from the stack.
    pub fn current_blame_context(&self) -> Option<BlameId> {
        let stack = self.blame_stack.lock().unwrap();
        stack.last().copied()
    }

    /// Pushes a blame context onto the stack.
    pub fn push_blame_context(&self, id: BlameId) {
        let mut stack = self.blame_stack.lock().unwrap();
        stack.push(id);
    }

    /// Pops a blame context from the stack.
    pub fn pop_blame_context(&self) -> Option<BlameId> {
        let mut stack = self.blame_stack.lock().unwrap();
        stack.pop()
    }

    /// Gets blame information by ID.
    pub fn get_blame_info(&self, id: BlameId) -> Option<BlameInfo> {
        let contexts = self.contexts.lock().unwrap();
        contexts.get(&id).cloned()
    }

    /// Records a contract violation.
    pub fn record_violation(
        &self,
        blame: BlameInfo,
        contract: String,
        expected: String,
        actual: String,
        message: String,
        location: Span,
    ) {
        let violation = BlameViolation {
            blame,
            contract,
            expected,
            actual,
            message,
            timestamp: std::time::SystemTime::now(),
            location,
        };

        let mut violations = self.violations.lock().unwrap();
        violations.push_back(violation);

        // Trim history if needed
        while violations.len() > self.max_history {
            violations.pop_front();
        }
    }

    /// Gets recent violations.
    pub fn recent_violations(&self, limit: usize) -> Vec<BlameViolation> {
        let violations = self.violations.lock().unwrap();
        violations.iter().rev().take(limit).cloned().collect()
    }

    /// Gets all violations for a specific blame context.
    pub fn violations_for_context(&self, blame_id: BlameId) -> Vec<BlameViolation> {
        let violations = self.violations.lock().unwrap();
        violations
            .iter()
            .filter(|v| v.blame.id == blame_id)
            .cloned()
            .collect()
    }

    /// Clears violation history.
    pub fn clear_history(&self) {
        let mut violations = self.violations.lock().unwrap();
        violations.clear();
    }

    /// Gets statistics about violations.
    pub fn violation_stats(&self) -> BlameStats {
        let violations = self.violations.lock().unwrap();
        let total_violations = violations.len();
        
        let mut by_target = HashMap::new();
        let mut by_contract = HashMap::new();
        
        for violation in violations.iter() {
            // Count by positive blame target
            let target_key = format!("{}", violation.blame.positive);
            *by_target.entry(target_key).or_insert(0) += 1;
            
            // Count by contract type
            *by_contract.entry(violation.contract.clone()).or_insert(0) += 1;
        }

        BlameStats {
            total_violations,
            violations_by_target: by_target,
            violations_by_contract: by_contract,
        }
    }
}

/// Statistics about blame violations.
#[derive(Debug, Clone)]
pub struct BlameStats {
    /// Total number of violations
    pub total_violations: usize,
    /// Violations grouped by blame target
    pub violations_by_target: HashMap<String, usize>,
    /// Violations grouped by contract type
    pub violations_by_contract: HashMap<String, usize>,
}

impl BlameInfo {
    /// Creates blame info for a function definition.
    pub fn function_definition(
        function_name: String,
        module: Option<String>,
        location: Span,
        contract: String,
    ) -> Self {
        let positive = BlameTarget::Function {
            name: "caller".to_string(),
            module: None,
            span: location,
        };
        
        let negative = BlameTarget::Function {
            name: function_name,
            module,
            span: location,
        };

        let boundary = BlameBoundary {
            boundary_type: BoundaryType::FunctionDefinition,
            contract,
            location,
            context: HashMap::new(),
        };

        BlameInfo {
            positive,
            negative,
            boundary,
            call_stack: Vec::new(),
            id: 0, // Will be assigned by BlameTracker
            parent: None,
        }
    }

    /// Creates blame info for module interface.
    pub fn module_interface(
        module_name: String,
        interface: String,
        location: Span,
        contract: String,
    ) -> Self {
        let positive = BlameTarget::Module {
            name: "client".to_string(),
            interface: interface.clone(),
        };
        
        let negative = BlameTarget::Module {
            name: module_name,
            interface,
        };

        let boundary = BlameBoundary {
            boundary_type: BoundaryType::ModuleInterface,
            contract,
            location,
            context: HashMap::new(),
        };

        BlameInfo {
            positive,
            negative,
            boundary,
            call_stack: Vec::new(),
            id: 0,
            parent: None,
        }
    }

    /// Swaps positive and negative blame (for higher-order contexts).
    pub fn swap_blame(&self) -> Self {
        BlameInfo {
            positive: self.negative.clone(),
            negative: self.positive.clone(),
            boundary: self.boundary.clone(),
            call_stack: self.call_stack.clone(),
            id: self.id,
            parent: self.parent,
        }
    }

    /// Adds context information to the boundary.
    pub fn with_context(mut self, key: String, value: String) -> Self {
        self.boundary.context.insert(key, value);
        self
    }
}

impl CallFrame {
    /// Creates a function call frame.
    pub fn function_call(name: String, location: Span, module: Option<String>) -> Self {
        Self {
            name,
            location,
            module,
            frame_type: FrameType::FunctionCall,
        }
    }

    /// Creates a contract check frame.
    pub fn contract_check(contract: String, location: Span) -> Self {
        Self {
            name: format!("contract-check({contract})"),
            location,
            module: None,
            frame_type: FrameType::ContractCheck,
        }
    }
}

impl fmt::Display for BlameTarget {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BlameTarget::Location { file, span, description } => {
                write!(f, "{}:{}:{} ({})", file, span.start, span.end(), description)
            }
            BlameTarget::Function { name, module, span } => {
                if let Some(module) = module {
                    write!(f, "{}:{} at {}:{}", module, name, span.start, span.end())
                } else {
                    write!(f, "{} at {}:{}", name, span.start, span.end())
                }
            }
            BlameTarget::Module { name, interface } => {
                write!(f, "module {name} ({interface})")
            }
            BlameTarget::Repl { input_number, span } => {
                write!(f, "REPL input #{input_number} at {}:{}", span.start, span.end())
            }
            BlameTarget::System { component, description } => {
                write!(f, "system {component} ({description})")
            }
            BlameTarget::Unknown { description } => {
                write!(f, "unknown ({description})")
            }
        }
    }
}

impl fmt::Display for BlameInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "positive: {}, negative: {}", self.positive, self.negative)
    }
}

impl Default for BlameTracker {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::diagnostics::Span;

    #[test]
    fn test_blame_tracker_creation() {
        let tracker = BlameTracker::new();
        assert_eq!(tracker.max_history, 1000);
        assert!(tracker.current_blame_context().is_none());
    }

    #[test]
    fn test_blame_id_generation() {
        let tracker = BlameTracker::new();
        let id1 = tracker.new_blame_id();
        let id2 = tracker.new_blame_id();
        assert_ne!(id1, id2);
        assert_eq!(id1 + 1, id2);
    }

    #[test]
    fn test_blame_context_stack() {
        let tracker = BlameTracker::new();
        
        let id1 = tracker.new_blame_id();
        let id2 = tracker.new_blame_id();
        
        tracker.push_blame_context(id1);
        assert_eq!(tracker.current_blame_context(), Some(id1));
        
        tracker.push_blame_context(id2);
        assert_eq!(tracker.current_blame_context(), Some(id2));
        
        assert_eq!(tracker.pop_blame_context(), Some(id2));
        assert_eq!(tracker.current_blame_context(), Some(id1));
        
        assert_eq!(tracker.pop_blame_context(), Some(id1));
        assert_eq!(tracker.current_blame_context(), None);
    }

    #[test]
    fn test_blame_info_creation() {
        let span = Span::new(0, 10);
        let blame = BlameInfo::function_definition(
            "test-function".to_string(),
            Some("test-module".to_string()),
            span,
            "number? -> string?".to_string(),
        );
        
        assert!(matches!(blame.positive, BlameTarget::Function { .. }));
        assert!(matches!(blame.negative, BlameTarget::Function { .. }));
        assert_eq!(blame.boundary.boundary_type, BoundaryType::FunctionDefinition);
    }

    #[test]
    fn test_violation_recording() {
        let tracker = BlameTracker::new();
        let span = Span::new(0, 10);
        
        let blame = BlameInfo::function_definition(
            "test-function".to_string(),
            None,
            span,
            "number?".to_string(),
        );
        
        tracker.record_violation(
            blame,
            "number?".to_string(),
            "number".to_string(),
            "string".to_string(),
            "Expected number, got string".to_string(),
            span,
        );
        
        let violations = tracker.recent_violations(10);
        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].expected, "number");
        assert_eq!(violations[0].actual, "string");
    }

    #[test]
    fn test_blame_swap() {
        let span = Span::new(0, 10);
        let blame = BlameInfo::function_definition(
            "test-function".to_string(),
            None,
            span,
            "number?".to_string(),
        );
        
        let swapped = blame.swap_blame();
        
        // Positive and negative should be swapped
        assert_eq!(format!("{}", blame.positive), format!("{}", swapped.negative));
        assert_eq!(format!("{}", blame.negative), format!("{}", swapped.positive));
    }
}