//! Debugger integration for the enhanced REPL system.

#![allow(dead_code, missing_docs)]

use crate::{Lambdust, Result, Error, eval::Value};
use std::collections::{HashMap, HashSet};

/// Debug commands that can be executed during debugging
#[derive(Debug, Clone, PartialEq)]
pub enum DebugCommand {
    Step,
    Continue,
    StepOver,
    StepOut,
    Break(String),
    RemoveBreak(String),
    ListBreaks,
    ShowStack,
    ShowVars,
    Evaluate(String),
}

/// Manages breakpoints in the debugger
#[derive(Debug, Clone)]
pub struct BreakpointManager {
    breakpoints: HashSet<String>,
    conditional_breakpoints: HashMap<String, String>,
}

impl Default for BreakpointManager {
    fn default() -> Self {
        Self::new()
    }
}

impl BreakpointManager {
    pub fn new() -> Self {
        Self {
            breakpoints: HashSet::new(),
            conditional_breakpoints: HashMap::new(),
        }
    }

    pub fn add_breakpoint(&mut self, expression: &str) {
        self.breakpoints.insert(expression.to_string());
    }

    pub fn remove_breakpoint(&mut self, expression: &str) -> bool {
        self.breakpoints.remove(expression)
    }

    pub fn add_conditional_breakpoint(&mut self, expression: &str, condition: &str) {
        self.breakpoints.insert(expression.to_string());
        self.conditional_breakpoints.insert(expression.to_string(), condition.to_string());
    }

    pub fn has_breakpoint(&self, expression: &str) -> bool {
        self.breakpoints.contains(expression)
    }

    pub fn should_break(&self, expression: &str, _context: &DebugContext) -> bool {
        if !self.has_breakpoint(expression) {
            return false;
        }

        // For conditional breakpoints, we would evaluate the condition here
        if let Some(_condition) = self.conditional_breakpoints.get(expression) {
            // TODO: Evaluate condition in current context
            true
        } else {
            true
        }
    }

    pub fn list_breakpoints(&self) -> Vec<String> {
        self.breakpoints.iter().cloned().collect()
    }

    pub fn clear_all(&mut self) {
        self.breakpoints.clear();
        self.conditional_breakpoints.clear();
    }
}

/// Debug context containing information about the current execution state
#[derive(Debug, Clone)]
pub struct DebugContext {
    pub call_stack: Vec<CallFrame>,
    pub current_expression: String,
    pub variables: HashMap<String, Value>,
    pub step_mode: StepMode,
}

/// Represents a frame in the call stack
#[derive(Debug, Clone)]
pub struct CallFrame {
    pub function_name: String,
    pub expression: String,
    pub local_vars: HashMap<String, Value>,
    pub line_number: Option<usize>,
}

/// Different stepping modes for the debugger
#[derive(Debug, Clone, PartialEq)]
pub enum StepMode {
    None,
    StepInto,
    StepOver,
    StepOut,
    Continue,
}

/// The main debugger implementation
#[derive(Debug)]
pub struct Debugger {
    enabled: bool,
    breakpoint_manager: BreakpointManager,
    debug_context: Option<DebugContext>,
    step_mode: StepMode,
    execution_paused: bool,
}

impl Debugger {
    pub fn new() -> Self {
        Self {
            enabled: false,
            breakpoint_manager: BreakpointManager::new(),
            debug_context: None,
            step_mode: StepMode::None,
            execution_paused: false,
        }
    }

    pub fn enable(&mut self) {
        self.enabled = true;
    }

    pub fn disable(&mut self) {
        self.enabled = false;
        self.execution_paused = false;
        self.step_mode = StepMode::None;
    }

    pub fn is_debugging(&self) -> bool {
        self.enabled
    }

    pub fn is_paused(&self) -> bool {
        self.execution_paused
    }

    pub fn set_breakpoint(&mut self, expression: &str) -> Result<()> {
        if !self.enabled {
            return Err(Box::new(Error::runtime_error("Debugger not enabled", None)));
        }
        
        self.breakpoint_manager.add_breakpoint(expression);
        Ok(())
    }

    pub fn remove_breakpoint(&mut self, expression: &str) -> Result<bool> {
        if !self.enabled {
            return Err(Box::new(Error::runtime_error("Debugger not enabled", None)));
        }
        
        Ok(self.breakpoint_manager.remove_breakpoint(expression))
    }

    pub fn list_breakpoints(&self) -> Vec<String> {
        self.breakpoint_manager.list_breakpoints()
    }

    pub fn step(&mut self) -> Result<()> {
        if !self.enabled {
            return Err(Box::new(Error::runtime_error("Debugger not enabled", None)));
        }
        
        self.step_mode = StepMode::StepInto;
        self.execution_paused = false;
        println!("Stepping into next expression...");
        Ok(())
    }

    pub fn step_over(&mut self) -> Result<()> {
        if !self.enabled {
            return Err(Box::new(Error::runtime_error("Debugger not enabled", None)));
        }
        
        self.step_mode = StepMode::StepOver;
        self.execution_paused = false;
        println!("Stepping over next expression...");
        Ok(())
    }

    pub fn step_out(&mut self) -> Result<()> {
        if !self.enabled {
            return Err(Box::new(Error::runtime_error("Debugger not enabled", None)));
        }
        
        self.step_mode = StepMode::StepOut;
        self.execution_paused = false;
        println!("Stepping out of current function...");
        Ok(())
    }

    pub fn continue_execution(&mut self) -> Result<()> {
        if !self.enabled {
            return Err(Box::new(Error::runtime_error("Debugger not enabled", None)));
        }
        
        self.step_mode = StepMode::Continue;
        self.execution_paused = false;
        println!("Continuing execution...");
        Ok(())
    }

    pub fn show_stack_trace(&self) -> Result<()> {
        if let Some(ref context) = self.debug_context {
            println!("Call Stack:");
            for (i, frame) in context.call_stack.iter().enumerate() {
                println!("  {}: {} in {}", 
                    i, 
                    frame.function_name, 
                    frame.expression
                );
                if let Some(line) = frame.line_number {
                    println!("      at line {line}");
                }
            }
        } else {
            println!("No debug context available");
        }
        Ok(())
    }

    pub fn show_variables(&self) -> Result<()> {
        if let Some(ref context) = self.debug_context {
            println!("Variables in current scope:");
            for (name, value) in &context.variables {
                println!("  {name} = {value}");
            }
            
            // Show local variables from current call frame
            if let Some(current_frame) = context.call_stack.last() {
                if !current_frame.local_vars.is_empty() {
                    println!("Local variables:");
                    for (name, value) in &current_frame.local_vars {
                        println!("  {name} = {value}");
                    }
                }
            }
        } else {
            println!("No debug context available");
        }
        Ok(())
    }

    pub fn evaluate_with_debug(&mut self, lambdust: &mut Lambdust, input: &str) -> Result<()> {
        if !self.enabled {
            return Err(Box::new(Error::runtime_error("Debugger not enabled", None)));
        }

        // Create a debug context for this evaluation
        let context = DebugContext {
            call_stack: vec![
                CallFrame {
                    function_name: "<repl>".to_string(),
                    expression: input.to_string(),
                    local_vars: HashMap::new(),
                    line_number: None,
                }
            ],
            current_expression: input.to_string(),
            variables: HashMap::new(),
            step_mode: self.step_mode.clone(),
        };

        // Check if we should break on this expression
        if self.breakpoint_manager.should_break(input, &context) {
            self.execution_paused = true;
            println!("🔴 Breakpoint hit: {input}");
            self.debug_context = Some(context);
            return Ok(());
        }

        // Evaluate with debug instrumentation
        self.debug_context = Some(context);
        
        match lambdust.eval(input, Some("<repl-debug>")) {
            Ok(result) => {
                println!("🟢 {result}");
                // Reset debug context after successful evaluation
                self.debug_context = None;
                Ok(())
            }
            Err(e) => {
                println!("🔴 Debug Error: {e}");
                // Keep debug context for inspection
                Ok(())
            }
        }
    }

    pub fn inspect_current_state(&self) -> Result<()> {
        if let Some(ref context) = self.debug_context {
            println!("🔍 Current Debug State:");
            println!("  Expression: {}", context.current_expression);
            println!("  Step Mode: {:?}", context.step_mode);
            println!("  Call Stack Depth: {}", context.call_stack.len());
            
            if let Some(current_frame) = context.call_stack.last() {
                println!("  Current Function: {}", current_frame.function_name);
            }
        } else {
            println!("No active debug session");
        }
        Ok(())
    }

    pub fn handle_debug_command(&mut self, command: DebugCommand) -> Result<()> {
        match command {
            DebugCommand::Step => self.step(),
            DebugCommand::Continue => self.continue_execution(),
            DebugCommand::StepOver => self.step_over(),
            DebugCommand::StepOut => self.step_out(),
            DebugCommand::Break(expr) => self.set_breakpoint(&expr),
            DebugCommand::RemoveBreak(expr) => {
                self.remove_breakpoint(&expr)?;
                println!("Breakpoint removed: {expr}");
                Ok(())
            }
            DebugCommand::ListBreaks => {
                let breakpoints = self.list_breakpoints();
                if breakpoints.is_empty() {
                    println!("No breakpoints set");
                } else {
                    println!("Breakpoints:");
                    for (i, bp) in breakpoints.iter().enumerate() {
                        println!("  {}: {}", i + 1, bp);
                    }
                }
                Ok(())
            }
            DebugCommand::ShowStack => self.show_stack_trace(),
            DebugCommand::ShowVars => self.show_variables(),
            DebugCommand::Evaluate(expr) => {
                // TODO: Evaluate expression in current debug context
                println!("Evaluating: {expr}");
                Ok(())
            }
        }
    }
}

impl Default for Debugger {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_breakpoint_manager() {
        let mut manager = BreakpointManager::new();
        
        // Test adding breakpoints
        manager.add_breakpoint("(+ 1 2)");
        assert!(manager.has_breakpoint("(+ 1 2)"));
        assert!(!manager.has_breakpoint("(* 3 4)"));
        
        // Test removing breakpoints
        assert!(manager.remove_breakpoint("(+ 1 2)"));
        assert!(!manager.has_breakpoint("(+ 1 2)"));
        assert!(!manager.remove_breakpoint("(+ 1 2)"));
    }

    #[test]
    fn test_debugger_state() {
        let mut debugger = Debugger::new();
        
        // Test initial state
        assert!(!debugger.is_debugging());
        assert!(!debugger.is_paused());
        
        // Test enabling
        debugger.enable();
        assert!(debugger.is_debugging());
        
        // Test disabling
        debugger.disable();
        assert!(!debugger.is_debugging());
        assert!(!debugger.is_paused());
    }
}