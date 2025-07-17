//! Position and range utilities for LSP integration
//!
//! This module provides position-aware parsing and text manipulation utilities
//! that enable precise location tracking for LSP features like diagnostics,
//! completion, and navigation.

use crate::error::{LambdustError, Result};
use std::ops::Range as StdRange;

/// Position in a text document (zero-indexed)
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Position {
    /// Line number (zero-indexed)
    pub line: u32,
    /// Character offset in line (zero-indexed, UTF-16 code units)
    pub character: u32,
}

/// Range in a text document
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Range {
    /// Start position (inclusive)
    pub start: Position,
    /// End position (exclusive)
    pub end: Position,
}

/// Span representing a source location with additional metadata
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Span {
    /// Range in the document
    pub range: Range,
    /// Source file path (if available)
    pub file: Option<String>,
    /// Additional context information
    pub context: Option<String>,
}

/// Position tracking during parsing and lexical analysis
#[derive(Debug, Clone)]
pub struct PositionTracker {
    /// Current line (zero-indexed)
    line: u32,
    /// Current character (zero-indexed)
    character: u32,
    /// Total offset from start of document
    offset: usize,
    /// Line start offsets for efficient position calculation
    line_starts: Vec<usize>,
}

/// Utilities for working with positions and ranges
pub struct PositionUtils;

impl Position {
    /// Create a new position
    pub const fn new(line: u32, character: u32) -> Self {
        Self { line, character }
    }
    
    /// Create position at start of document
    pub const fn zero() -> Self {
        Self::new(0, 0)
    }
    
    /// Check if position is valid
    pub fn is_valid(&self) -> bool {
        self.line < u32::MAX && self.character < u32::MAX
    }
    
    /// Calculate UTF-8 byte offset in a line
    pub fn to_utf8_offset(&self, line_text: &str) -> Option<usize> {
        let mut utf16_count = 0;
        let mut utf8_offset = 0;
        
        for ch in line_text.chars() {
            if utf16_count >= self.character {
                return Some(utf8_offset);
            }
            utf16_count += ch.len_utf16() as u32;
            utf8_offset += ch.len_utf8();
        }
        
        if utf16_count == self.character {
            Some(utf8_offset)
        } else {
            None
        }
    }
}

impl Range {
    /// Create a new range
    pub const fn new(start: Position, end: Position) -> Self {
        Self { start, end }
    }
    
    /// Create a range from start position with length
    pub fn from_length(start: Position, length: u32) -> Self {
        Self::new(
            start,
            Position::new(start.line, start.character + length)
        )
    }
    
    /// Create a zero-width range at position
    pub fn at_position(pos: Position) -> Self {
        Self::new(pos, pos)
    }
    
    /// Check if range contains position
    pub fn contains(&self, pos: Position) -> bool {
        self.start <= pos && pos < self.end
    }
    
    /// Check if range contains another range
    pub fn contains_range(&self, other: &Range) -> bool {
        self.start <= other.start && other.end <= self.end
    }
    
    /// Check if ranges overlap
    pub fn overlaps(&self, other: &Range) -> bool {
        self.start < other.end && other.start < self.end
    }
    
    /// Get intersection of two ranges
    pub fn intersection(&self, other: &Range) -> Option<Range> {
        let start = self.start.max(other.start);
        let end = self.end.min(other.end);
        
        if start < end {
            Some(Range::new(start, end))
        } else {
            None
        }
    }
    
    /// Check if range is valid
    pub fn is_valid(&self) -> bool {
        self.start <= self.end && 
        self.start.is_valid() && 
        self.end.is_valid()
    }
    
    /// Check if range is empty (zero-width)
    pub fn is_empty(&self) -> bool {
        self.start == self.end
    }
}

impl Span {
    /// Create a new span
    pub fn new(range: Range) -> Self {
        Self {
            range,
            file: None,
            context: None,
        }
    }
    
    /// Create span with file information
    pub fn with_file(range: Range, file: String) -> Self {
        Self {
            range,
            file: Some(file),
            context: None,
        }
    }
    
    /// Create span with context information
    pub fn with_context(range: Range, context: String) -> Self {
        Self {
            range,
            file: None,
            context: Some(context),
        }
    }
    
    /// Create span with both file and context
    pub fn with_file_and_context(range: Range, file: String, context: String) -> Self {
        Self {
            range,
            file: Some(file),
            context: Some(context),
        }
    }
}

impl PositionTracker {
    /// Create a new position tracker
    pub fn new() -> Self {
        Self {
            line: 0,
            character: 0,
            offset: 0,
            line_starts: vec![0],
        }
    }
    
    /// Get current position
    pub fn current_position(&self) -> Position {
        Position::new(self.line, self.character)
    }
    
    /// Get current offset
    pub fn current_offset(&self) -> usize {
        self.offset
    }
    
    /// Advance by one character
    pub fn advance_char(&mut self, ch: char) {
        self.offset += ch.len_utf8();
        
        if ch == '\n' {
            self.line += 1;
            self.character = 0;
            self.line_starts.push(self.offset);
        } else {
            self.character += ch.len_utf16() as u32;
        }
    }
    
    /// Advance by string
    pub fn advance_str(&mut self, s: &str) {
        for ch in s.chars() {
            self.advance_char(ch);
        }
    }
    
    /// Convert offset to position
    pub fn offset_to_position(&self, offset: usize) -> Option<Position> {
        if offset > self.offset {
            return None;
        }
        
        // Binary search for line
        let line_idx = match self.line_starts.binary_search(&offset) {
            Ok(idx) => idx,
            Err(idx) => idx.saturating_sub(1),
        };
        
        if line_idx >= self.line_starts.len() {
            return None;
        }
        
        let line_start = self.line_starts[line_idx];
        let line_offset = offset - line_start;
        
        // Convert byte offset to UTF-16 character offset
        // For this, we'd need the actual line text - simplified for now
        Some(Position::new(line_idx as u32, line_offset as u32))
    }
    
    /// Convert position to offset
    pub fn position_to_offset(&self, pos: Position) -> Option<usize> {
        if pos.line as usize >= self.line_starts.len() {
            return None;
        }
        
        let line_start = self.line_starts[pos.line as usize];
        Some(line_start + pos.character as usize)
    }
    
    /// Reset tracker state
    pub fn reset(&mut self) {
        self.line = 0;
        self.character = 0;
        self.offset = 0;
        self.line_starts.clear();
        self.line_starts.push(0);
    }
}

impl PositionUtils {
    /// Extract text in range from document
    pub fn extract_text(text: &str, range: Range) -> Result<String> {
        let lines: Vec<&str> = text.lines().collect();
        
        if range.start.line as usize >= lines.len() {
            return Err(LambdustError::runtime_error(
                format!("Start line {} out of bounds", range.start.line)
            ));
        }
        
        if range.end.line as usize >= lines.len() {
            return Err(LambdustError::runtime_error(
                format!("End line {} out of bounds", range.end.line)
            ));
        }
        
        if range.start.line == range.end.line {
            // Single line
            let line = lines[range.start.line as usize];
            let start_offset = range.start.to_utf8_offset(line)
                .ok_or_else(|| LambdustError::runtime_error("Invalid start position"))?;
            let end_offset = range.end.to_utf8_offset(line)
                .ok_or_else(|| LambdustError::runtime_error("Invalid end position"))?;
            
            if start_offset <= end_offset && end_offset <= line.len() {
                Ok(line[start_offset..end_offset].to_string())
            } else {
                Err(LambdustError::runtime_error("Invalid range"))
            }
        } else {
            // Multi-line
            let mut result = String::new();
            
            // First line
            let first_line = lines[range.start.line as usize];
            let start_offset = range.start.to_utf8_offset(first_line)
                .ok_or_else(|| LambdustError::runtime_error("Invalid start position"))?;
            result.push_str(&first_line[start_offset..]);
            result.push('\n');
            
            // Middle lines
            for line_idx in (range.start.line + 1)..range.end.line {
                result.push_str(lines[line_idx as usize]);
                result.push('\n');
            }
            
            // Last line
            let last_line = lines[range.end.line as usize];
            let end_offset = range.end.to_utf8_offset(last_line)
                .ok_or_else(|| LambdustError::runtime_error("Invalid end position"))?;
            result.push_str(&last_line[..end_offset]);
            
            Ok(result)
        }
    }
    
    /// Find position of substring in text
    pub fn find_position(text: &str, substring: &str) -> Vec<Position> {
        let mut positions = Vec::new();
        let mut tracker = PositionTracker::new();
        
        let bytes = text.as_bytes();
        let pattern = substring.as_bytes();
        
        let mut pos = 0;
        while pos < bytes.len() {
            if bytes[pos..].starts_with(pattern) {
                positions.push(tracker.current_position());
                pos += pattern.len();
                tracker.advance_str(substring);
            } else {
                let ch = text.chars().nth(tracker.current_offset()).unwrap_or('\0');
                tracker.advance_char(ch);
                pos += ch.len_utf8();
            }
        }
        
        positions
    }
    
    /// Convert between different position representations
    pub fn lsp_position_to_position(lsp_pos: &lsp_types::Position) -> Position {
        Position::new(lsp_pos.line, lsp_pos.character)
    }
    
    pub fn position_to_lsp_position(pos: Position) -> lsp_types::Position {
        lsp_types::Position::new(pos.line, pos.character)
    }
    
    pub fn lsp_range_to_range(lsp_range: &lsp_types::Range) -> Range {
        Range::new(
            Self::lsp_position_to_position(&lsp_range.start),
            Self::lsp_position_to_position(&lsp_range.end),
        )
    }
    
    pub fn range_to_lsp_range(range: Range) -> lsp_types::Range {
        lsp_types::Range::new(
            Self::position_to_lsp_position(range.start),
            Self::position_to_lsp_position(range.end),
        )
    }
}

impl Default for PositionTracker {
    fn default() -> Self {
        Self::new()
    }
}
