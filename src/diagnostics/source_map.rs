//! Source mapping and position tracking for error reporting.

use super::{Span, Position};
use std::collections::HashMap;
use std::sync::Arc;

/// Maps source code to line/column positions for error reporting.
#[derive(Debug, Clone)]
pub struct SourceMap {
    /// Source file name
    pub filename: String,
    /// Complete source text
    pub source: String,
    /// Line start positions (byte offsets)
    line_starts: Vec<usize>,
    /// File ID for this source
    pub file_id: usize,
}

impl SourceMap {
    /// Creates a new source map from source text.
    pub fn new(filename: String, source: String, file_id: usize) -> Self {
        let mut line_starts = vec![0];
        
        for (i, ch) in source.char_indices() {
            if ch == '\n' {
                line_starts.push(i + 1);
            }
        }
        
        Self {
            filename,
            source,
            line_starts,
            file_id,
        }
    }
    
    /// Converts a byte offset to line/column position.
    pub fn position_at_offset(&self, offset: usize) -> Position {
        let line = self.line_starts
            .binary_search(&offset)
            .unwrap_or_else(|i| i.saturating_sub(1));
        
        let line_start = self.line_starts.get(line).copied().unwrap_or(0);
        let column = offset.saturating_sub(line_start) + 1;
        
        Position::new(line + 1, column, offset)
    }
    
    /// Creates a span with position information.
    pub fn span_with_position(&self, start: usize, len: usize) -> Span {
        let _pos = self.position_at_offset(start);
        Span::with_file(start, len, self.file_id)
    }
    
    /// Gets the source text for a span.
    pub fn source_text(&self, span: &Span) -> &str {
        let start = span.start.min(self.source.len());
        let end = span.end().min(self.source.len());
        &self.source[start..end]
    }
    
    /// Gets the line text containing a span.
    pub fn line_text(&self, span: &Span) -> &str {
        let pos = self.position_at_offset(span.start);
        let line_idx = pos.line.saturating_sub(1);
        
        if line_idx < self.line_starts.len() {
            let line_start = self.line_starts[line_idx];
            let line_end = self.line_starts
                .get(line_idx + 1)
                .copied()
                .unwrap_or(self.source.len());
            
            let line_text = &self.source[line_start..line_end];
            line_text.trim_end_matches('\n')
        } else {
            ""
        }
    }
    
    /// Gets context lines around a span for error display.
    pub fn context_lines(&self, span: &Span, context_size: usize) -> Vec<(usize, String)> {
        let pos = self.position_at_offset(span.start);
        let center_line = pos.line.saturating_sub(1);
        
        let start_line = center_line.saturating_sub(context_size);
        let end_line = (center_line + context_size + 1).min(self.line_starts.len());
        
        (start_line..end_line)
            .map(|line_idx| {
                let line_start = self.line_starts[line_idx];
                let line_end = self.line_starts
                    .get(line_idx + 1)
                    .copied()
                    .unwrap_or(self.source.len());
                
                let line_text = self.source[line_start..line_end]
                    .trim_end_matches('\n')
                    .to_string();
                
                (line_idx + 1, line_text)
            })
            .collect()
    }
    
    /// Calculates column indicators for underlining errors.
    pub fn column_indicators(&self, span: &Span) -> (usize, usize) {
        let start_pos = self.position_at_offset(span.start);
        let end_pos = self.position_at_offset(span.end());
        
        if start_pos.line == end_pos.line {
            (start_pos.column, end_pos.column)
        } else {
            // Multi-line span, just show start column
            (start_pos.column, start_pos.column + span.len)
        }
    }
    
    /// Gets the number of lines in the source.
    pub fn line_count(&self) -> usize {
        self.line_starts.len()
    }
    
    /// Checks if an offset is valid for this source.
    pub fn is_valid_offset(&self, offset: usize) -> bool {
        offset <= self.source.len()
    }
}

/// Registry for managing multiple source files.
#[derive(Debug, Clone, Default)]
pub struct SourceRegistry {
    sources: HashMap<usize, Arc<SourceMap>>,
    next_file_id: usize,
}

impl SourceRegistry {
    /// Creates a new source registry.
    pub fn new() -> Self {
        Self {
            sources: HashMap::new(),
            next_file_id: 1,
        }
    }
    
    /// Registers a new source file.
    pub fn register_source(&mut self, filename: String, source: String) -> usize {
        let file_id = self.next_file_id;
        self.next_file_id += 1;
        
        let source_map = Arc::new(SourceMap::new(filename, source, file_id));
        self.sources.insert(file_id, source_map);
        
        file_id
    }
    
    /// Gets a source map by file ID.
    pub fn get_source(&self, file_id: usize) -> Option<Arc<SourceMap>> {
        self.sources.get(&file_id).clone())()
    }
    
    /// Gets all registered sources.
    pub fn all_sources(&self) -> impl Iterator<Item = (usize, Arc<SourceMap>)> + '_ {
        self.sources.iter().map(|(&id, source)| (id, source.clone()))
    }
    
    /// Removes a source file from the registry.
    pub fn remove_source(&mut self, file_id: usize) -> Option<Arc<SourceMap>> {
        self.sources.remove(&file_id)
    }
    
    /// Clears all registered sources.
    pub fn clear(&mut self) {
        self.sources.clear();
        self.next_file_id = 1;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_source_map_creation() {
        let source = "line 1\nline 2\nline 3".to_string();
        let map = SourceMap::new("test.scm".to_string(), source, 1);
        
        assert_eq!(map.line_count(), 3);
        assert_eq!(map.line_starts, vec![0, 7, 14]);
    }
    
    #[test]
    fn test_position_calculation() {
        let source = "line 1\nline 2\nline 3".to_string();
        let map = SourceMap::new("test.scm".to_string(), source, 1);
        
        // First line, first character
        let pos1 = map.position_at_offset(0);
        assert_eq!(pos1.line, 1);
        assert_eq!(pos1.column, 1);
        
        // Second line, first character
        let pos2 = map.position_at_offset(7);
        assert_eq!(pos2.line, 2);
        assert_eq!(pos2.column, 1);
        
        // Second line, third character
        let pos3 = map.position_at_offset(9);
        assert_eq!(pos3.line, 2);
        assert_eq!(pos3.column, 3);
    }
    
    #[test]
    fn test_span_with_position() {
        let source = "line 1\nline 2\nline 3".to_string();
        let map = SourceMap::new("test.scm".to_string(), source, 1);
        
        let span = map.span_with_position(7, 4); // "line" in line 2
        assert_eq!(span.start, 7);
        assert_eq!(span.len, 4);
        assert_eq!(span.line, 2);
        assert_eq!(span.column, 1);
        assert_eq!(span.file_id, Some(1));
    }
    
    #[test]
    fn test_source_text() {
        let source = "hello world".to_string();
        let map = SourceMap::new("test.scm".to_string(), source, 1);
        
        let span = Span::new(6, 5); // "world"
        assert_eq!(map.source_text(&span), "world");
    }
    
    #[test]
    fn test_line_text() {
        let source = "line 1\nline 2\nline 3".to_string();
        let map = SourceMap::new("test.scm".to_string(), source, 1);
        
        let span = Span::new(8, 3); // "ine" in line 2
        assert_eq!(map.line_text(&span), "line 2");
    }
    
    #[test]
    fn test_context_lines() {
        let source = "line 1\nline 2\nline 3\nline 4\nline 5".to_string();
        let map = SourceMap::new("test.scm".to_string(), source, 1);
        
        let span = Span::new(14, 4); // "line" in line 3
        let context = map.context_lines(&span, 1);
        
        assert_eq!(context.len(), 3);
        assert_eq!(context[0], (2, "line 2".to_string()));
        assert_eq!(context[1], (3, "line 3".to_string()));
        assert_eq!(context[2], (4, "line 4".to_string()));
    }
    
    #[test]
    fn test_column_indicators() {
        let source = "hello world".to_string();
        let map = SourceMap::new("test.scm".to_string(), source, 1);
        
        let span = Span::new(6, 5); // "world"
        let (start_col, end_col) = map.column_indicators(&span);
        
        assert_eq!(start_col, 7); // 1-based
        assert_eq!(end_col, 11);
    }
    
    #[test]
    fn test_source_registry() {
        let mut registry = SourceRegistry::new();
        
        let file1_id = registry.register_source("file1.scm".to_string(), "content 1".to_string());
        let file2_id = registry.register_source("file2.scm".to_string(), "content 2".to_string());
        
        assert_eq!(file1_id, 1);
        assert_eq!(file2_id, 2);
        
        let source1 = registry.get_source(file1_id).unwrap();
        assert_eq!(source1.filename, "file1.scm");
        assert_eq!(source1.source, "content 1");
        
        let source2 = registry.get_source(file2_id).unwrap();
        assert_eq!(source2.filename, "file2.scm");
        assert_eq!(source2.source, "content 2");
    }
}