//! Call stack tracing for runtime error reporting.

use super::Span;
use std::fmt;

/// Represents a single frame in the call stack.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CallFrame {
    /// Function or procedure name
    pub function_name: String,
    /// Source location where the call was made
    pub call_site: Option<Span>,
    /// Source location of the function definition
    pub definition_site: Option<Span>,
    /// File where the call occurred
    pub file_name: Option<String>,
    /// Additional context information
    pub context: Option<String>,
}

impl CallFrame {
    /// Creates a new call frame.
    pub fn new(function_name: impl Into<String>) -> Self {
        Self {
            function_name: function_name.into(),
            call_site: None,
            definition_site: None,
            file_name: None,
            context: None,
        }
    }
    
    /// Creates a call frame with call site information.
    pub fn with_call_site(mut self, call_site: Span) -> Self {
        self.call_site = Some(call_site);
        self
    }
    
    /// Creates a call frame with definition site information.
    pub fn with_definition_site(mut self, definition_site: Span) -> Self {
        self.definition_site = Some(definition_site);
        self
    }
    
    /// Creates a call frame with file information.
    pub fn with_file_name(mut self, file_name: impl Into<String>) -> Self {
        self.file_name = Some(file_name.into())
        self
    }
    
    /// Creates a call frame with context information.
    pub fn with_context(mut self, context: impl Into<String>) -> Self {
        self.context = Some(context.into())
        self
    }
    
    /// Returns true if this frame has location information.
    pub fn has_location(&self) -> bool {
        self.call_site.is_some() || self.definition_site.is_some()
    }
    
    /// Gets the primary span for this frame (call site if available, otherwise definition site).
    pub fn primary_span(&self) -> Option<Span> {
        self.call_site.or(self.definition_site)
    }
}

impl fmt::Display for CallFrame {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "  at {}", self.function_name)?;
        
        if let Some(file) = &self.file_name {
            write!(f, " in {}", file)?;
        }
        
        if let Some(call_site) = &self.call_site {
            write!(f, " (byte offset {})", call_site.start)?;
        } else if let Some(def_site) = &self.definition_site {
            write!(f, " (defined at byte offset {})", def_site.start)?;
        }
        
        if let Some(context) = &self.context {
            write!(f, " [{}]", context)?;
        }
        
        Ok(())
    }
}

/// Represents a complete call stack for error reporting.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CallStack {
    /// Stack frames from most recent to oldest
    pub frames: Vec<CallFrame>,
    /// Maximum number of frames to display
    pub max_display_frames: usize,
}

impl CallStack {
    /// Creates a new empty call stack.
    pub fn new() -> Self {
        Self {
            frames: Vec::new(),
            max_display_frames: 10,
        }
    }
    
    /// Creates a call stack with a maximum display limit.
    pub fn with_limit(max_display_frames: usize) -> Self {
        Self {
            frames: Vec::new(),
            max_display_frames,
        }
    }
    
    /// Pushes a new frame onto the stack.
    pub fn push(&mut self, frame: CallFrame) {
        self.frames.push(frame);
    }
    
    /// Pops the most recent frame from the stack.
    pub fn pop(&mut self) -> Option<CallFrame> {
        self.frames.pop()
    }
    
    /// Gets the current stack depth.
    pub fn depth(&self) -> usize {
        self.frames.len()
    }
    
    /// Checks if the stack is empty.
    pub fn is_empty(&self) -> bool {
        self.frames.is_empty()
    }
    
    /// Gets the most recent frame.
    pub fn top(&self) -> Option<&CallFrame> {
        self.frames.last()
    }
    
    /// Gets all frames in reverse order (most recent first).
    pub fn frames(&self) -> impl Iterator<Item = &CallFrame> {
        self.frames.iter().rev()
    }
    
    /// Gets frames for display, respecting the maximum display limit.
    pub fn display_frames(&self) -> impl Iterator<Item = &CallFrame> {
        let skip_count = if self.frames.len() > self.max_display_frames {
            self.frames.len() - self.max_display_frames
        } else {
            0
        };
        
        self.frames.iter().rev().skip(skip_count)
    }
    
    /// Returns true if there are more frames than can be displayed.
    pub fn has_hidden_frames(&self) -> bool {
        self.frames.len() > self.max_display_frames
    }
    
    /// Gets the number of hidden frames.
    pub fn hidden_frame_count(&self) -> usize {
        if self.frames.len() > self.max_display_frames {
            self.frames.len() - self.max_display_frames
        } else {
            0
        }
    }
    
    /// Clears all frames from the stack.
    pub fn clear(&mut self) {
        self.frames.clear();
    }
    
    /// Creates a new call stack with a single frame.
    pub fn single(frame: CallFrame) -> Self {
        let mut stack = Self::new();
        stack.push(frame);
        stack
    }
    
    /// Finds a frame by function name.
    pub fn find_frame(&self, function_name: &str) -> Option<&CallFrame> {
        self.frames.iter().find(|frame| frame.function_name == function_name)
    }
    
    /// Gets all frames with location information.
    pub fn frames_with_location(&self) -> impl Iterator<Item = &CallFrame> {
        self.frames.iter().filter(|frame| frame.has_location())
    }
    
    /// Formats the stack trace with line numbers and context.
    pub fn format_detailed(&self) -> String {
        if self.frames.is_empty() {
            return "  <no stack trace available>".to_string();
        }
        
        let mut result = String::new();
        
        if self.has_hidden_frames() {
            result.push_str(&format!("  ... {} more frames ...\n", self.hidden_frame_count()));
        }
        
        for frame in self.display_frames() {
            result.push_str(&format!("{}\n", frame));
        }
        
        result.trim_end().to_string()
    }
}

impl Default for CallStack {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for CallStack {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.frames.is_empty() {
            write!(f, "<no stack trace available>")
        } else {
            write!(f, "{}", self.format_detailed())
        }
    }
}

/// Builder for constructing call stacks incrementally.
#[derive(Debug)]
pub struct CallStackBuilder {
    stack: CallStack,
}

impl CallStackBuilder {
    /// Creates a new call stack builder.
    pub fn new() -> Self {
        Self {
            stack: CallStack::new(),
        }
    }
    
    /// Creates a builder with a display limit.
    pub fn with_limit(max_display_frames: usize) -> Self {
        Self {
            stack: CallStack::with_limit(max_display_frames),
        }
    }
    
    /// Adds a frame to the stack.
    pub fn frame(mut self, frame: CallFrame) -> Self {
        self.stack.push(frame);
        self
    }
    
    /// Adds a simple frame with just a function name.
    pub fn simple_frame(mut self, function_name: impl Into<String>) -> Self {
        self.stack.push(CallFrame::new(function_name));
        self
    }
    
    /// Adds a frame with location information.
    pub fn frame_with_location(
        mut self,
        function_name: impl Into<String>,
        call_site: Span,
        file_name: Option<String>,
    ) -> Self {
        let mut frame = CallFrame::new(function_name);
        frame.call_site = Some(call_site);
        frame.file_name = file_name;
        self.stack.push(frame);
        self
    }
    
    /// Builds the final call stack.
    pub fn build(self) -> CallStack {
        self.stack
    }
}

impl Default for CallStackBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_call_frame_creation() {
        let frame = CallFrame::new("test_function");
        assert_eq!(frame.function_name, "test_function");
        assert!(frame.call_site.is_none());
        assert!(frame.definition_site.is_none());
        assert!(!frame.has_location());
    }
    
    #[test]
    fn test_call_frame_with_location() {
        let span = Span::with_position(10, 5, 2, 15);
        let frame = CallFrame::new("test_function")
            .with_call_site(span)
            .with_file_name("test.scm");
        
        assert_eq!(frame.function_name, "test_function");
        assert_eq!(frame.call_site, Some(span));
        assert_eq!(frame.file_name, Some("test.scm".to_string()));
        assert!(frame.has_location());
        assert_eq!(frame.primary_span(), Some(span));
    }
    
    #[test]
    fn test_call_stack_operations() {
        let mut stack = CallStack::new();
        assert!(stack.is_empty());
        assert_eq!(stack.depth(), 0);
        
        let frame1 = CallFrame::new("function1");
        let frame2 = CallFrame::new("function2");
        
        stack.push(frame1.clone());
        stack.push(frame2.clone());
        
        assert!(!stack.is_empty());
        assert_eq!(stack.depth(), 2);
        assert_eq!(stack.top(), Some(&frame2));
        
        let popped = stack.pop();
        assert_eq!(popped, Some(frame2));
        assert_eq!(stack.depth(), 1);
        assert_eq!(stack.top(), Some(&frame1));
    }
    
    #[test]
    fn test_call_stack_display() {
        let mut stack = CallStack::new();
        
        let span1 = Span::with_position(10, 5, 1, 10);
        let span2 = Span::with_position(20, 8, 2, 5);
        
        let frame1 = CallFrame::new("main")
            .with_call_site(span1)
            .with_file_name("main.scm");
            
        let frame2 = CallFrame::new("helper")
            .with_call_site(span2)
            .with_file_name("helper.scm");
        
        stack.push(frame1);
        stack.push(frame2);
        
        let display = stack.format_detailed();
        assert!(display.contains("helper"));
        assert!(display.contains("main"));
        assert!(display.contains("line 2, column 5"));
        assert!(display.contains("line 1, column 10"));
    }
    
    #[test]
    fn test_call_stack_limit() {
        let mut stack = CallStack::with_limit(2);
        
        for i in 1..=5 {
            stack.push(CallFrame::new(format!("function{}", i)));
        }
        
        assert_eq!(stack.depth(), 5);
        assert!(stack.has_hidden_frames());
        assert_eq!(stack.hidden_frame_count(), 3);
        
        let display_frames: Vec<_> = stack.display_frames().collect();
        assert_eq!(display_frames.len(), 2);
    }
    
    #[test]
    fn test_call_stack_builder() {
        let span = Span::with_position(15, 6, 3, 8);
        
        let stack = CallStackBuilder::new()
            .simple_frame("main")
            .frame_with_location("process", span, Some("process.scm".to_string()))
            .frame(CallFrame::new("helper").with_context("error handling"))
            .build();
        
        assert_eq!(stack.depth(), 3);
        
        let frames: Vec<_> = stack.frames().collect();
        assert_eq!(frames[0].function_name, "helper");
        assert_eq!(frames[1].function_name, "process");
        assert_eq!(frames[2].function_name, "main");
    }
    
    #[test]
    fn test_find_frame() {
        let mut stack = CallStack::new();
        stack.push(CallFrame::new("main"));
        stack.push(CallFrame::new("helper"));
        stack.push(CallFrame::new("process"));
        
        let found = stack.find_frame("helper");
        assert!(found.is_some());
        assert_eq!(found.unwrap().function_name, "helper");
        
        let not_found = stack.find_frame("nonexistent");
        assert!(not_found.is_none());
    }
    
    #[test]
    fn test_frames_with_location() {
        let mut stack = CallStack::new();
        
        let span = Span::with_position(10, 5, 1, 1);
        stack.push(CallFrame::new("no_location"));
        stack.push(CallFrame::new("with_location").with_call_site(span));
        stack.push(CallFrame::new("also_no_location"));
        
        let with_location: Vec<_> = stack.frames_with_location().collect();
        assert_eq!(with_location.len(), 1);
        assert_eq!(with_location[0].function_name, "with_location");
    }
}