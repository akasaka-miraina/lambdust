//! Document management for LSP text synchronization
//!
//! This module manages document lifecycle, text changes, and provides
//! utilities for working with document content in the LSP server.

use crate::error::{LambdustError, Result};
use crate::lsp::position::{Position, Range, PositionTracker};
use std::collections::HashMap;
use std::path::PathBuf;
use url::Url;

/// Document manager for tracking open documents
pub struct DocumentManager {
    /// Map of URI to document
    documents: HashMap<String, Document>,
    
    /// Document version tracking
    versions: HashMap<String, i32>,
    
    /// Document change tracking
    change_tracker: ChangeTracker,
}

/// Individual document representation
#[derive(Debug, Clone)]
pub struct Document {
    /// Document URI
    pub uri: String,
    
    /// Language identifier
    pub language_id: String,
    
    /// Document version
    pub version: i32,
    
    /// Document content
    content: String,
    
    /// Line endings style
    line_endings: LineEndings,
    
    /// Position tracker for efficient position calculations
    position_tracker: PositionTracker,
    
    /// Cached line starts for fast position lookup
    line_starts: Vec<usize>,
    
    /// Document metadata
    metadata: DocumentMetadata,
}

/// Document metadata
#[derive(Debug, Clone, Default)]
pub struct DocumentMetadata {
    /// File path (if available)
    pub file_path: Option<PathBuf>,
    
    /// Last modification time
    pub last_modified: Option<std::time::SystemTime>,
    
    /// Document size in bytes
    pub size: usize,
    
    /// Number of lines
    pub line_count: usize,
    
    /// Document encoding
    pub encoding: String,
}

/// Line ending styles
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LineEndings {
    /// Unix-style (\n)
    Unix,
    
    /// Windows-style (\r\n)
    Windows,
    
    /// Classic Mac-style (\r)
    Mac,
    
    /// Mixed line endings
    Mixed,
}

/// Change tracking for incremental updates
#[derive(Debug, Clone)]
pub struct ChangeTracker {
    /// Recent changes for each document
    changes: HashMap<String, Vec<DocumentChange>>,
    
    /// Maximum number of changes to track
    max_changes: usize,
}

/// Individual document change
#[derive(Debug, Clone)]
pub struct DocumentChange {
    /// Range that was changed
    pub range: Option<Range>,
    
    /// Length of replaced text
    pub range_length: Option<usize>,
    
    /// New text
    pub text: String,
    
    /// Timestamp of change
    pub timestamp: std::time::Instant,
}

impl DocumentManager {
    /// Create a new document manager
    pub fn new() -> Self {
        Self {
            documents: HashMap::new(),
            versions: HashMap::new(),
            change_tracker: ChangeTracker::new(),
        }
    }
    
    /// Handle document open event
    pub async fn did_open(&mut self, text_document: lsp_types::TextDocumentItem) -> Result<()> {
        let uri = text_document.uri.to_string();
        
        let document = Document::new(
            uri.clone(),
            text_document.language_id,
            text_document.version,
            text_document.text,
        )?;
        
        self.documents.insert(uri.clone(), document);
        self.versions.insert(uri, text_document.version);
        
        Ok(())
    }
    
    /// Handle document change event
    pub async fn did_change(
        &mut self,
        text_document: lsp_types::VersionedTextDocumentIdentifier,
        content_changes: Vec<lsp_types::TextDocumentContentChangeEvent>,
    ) -> Result<()> {
        let uri = text_document.uri.to_string();
        
        let document = self.documents.get_mut(&uri)
            .ok_or_else(|| LambdustError::runtime_error(format!("Document not found: {}", uri)))?;
        
        // Update version
        document.version = text_document.version;
        self.versions.insert(uri.clone(), text_document.version);
        
        // Apply changes
        for change in content_changes {
            self.apply_change(document, change)?;
        }
        
        // Update metadata
        document.update_metadata();
        
        Ok(())
    }
    
    /// Handle document close event
    pub async fn did_close(&mut self, uri: &lsp_types::Url) -> Result<()> {
        let uri_str = uri.to_string();
        self.documents.remove(&uri_str);
        self.versions.remove(&uri_str);
        self.change_tracker.clear_changes(&uri_str);
        
        Ok(())
    }
    
    /// Get document by URI
    pub fn get_document(&self, uri: &lsp_types::Url) -> Option<&Document> {
        self.documents.get(&uri.to_string())
    }
    
    /// Get mutable document by URI
    pub fn get_document_mut(&mut self, uri: &lsp_types::Url) -> Option<&mut Document> {
        self.documents.get_mut(&uri.to_string())
    }
    
    /// Get document version
    pub fn get_version(&self, uri: &lsp_types::Url) -> Option<i32> {
        self.versions.get(&uri.to_string()).copied()
    }
    
    /// List all open documents
    pub fn list_documents(&self) -> Vec<String> {
        self.documents.keys().cloned().collect()
    }
    
    /// Apply a content change to a document
    fn apply_change(
        &mut self,
        document: &mut Document,
        change: lsp_types::TextDocumentContentChangeEvent,
    ) -> Result<()> {
        use crate::lsp::position::PositionUtils;
        
        match change.range {
            Some(lsp_range) => {
                // Incremental change
                let range = PositionUtils::lsp_range_to_range(&lsp_range);
                
                // Extract the text to replace
                let start_offset = document.position_to_offset(range.start)?;
                let end_offset = document.position_to_offset(range.end)?;
                
                // Replace the text
                let mut content = document.content.clone();
                content.replace_range(start_offset..end_offset, &change.text);
                document.set_content(content)?;
                
                // Track the change
                self.change_tracker.add_change(&document.uri, DocumentChange {
                    range: Some(range),
                    range_length: change.range_length.map(|len| len as usize),
                    text: change.text,
                    timestamp: std::time::Instant::now(),
                });
            },
            None => {
                // Full document replacement
                document.set_content(change.text.clone())?;
                
                // Track the change
                self.change_tracker.add_change(&document.uri, DocumentChange {
                    range: None,
                    range_length: None,
                    text: change.text,
                    timestamp: std::time::Instant::now(),
                });
            }
        }
        
        Ok(())
    }
}

impl Document {
    /// Create a new document
    pub fn new(uri: String, language_id: String, version: i32, content: String) -> Result<Self> {
        let mut document = Self {
            uri,
            language_id,
            version,
            content: String::new(),
            line_endings: LineEndings::Unix,
            position_tracker: PositionTracker::new(),
            line_starts: Vec::new(),
            metadata: DocumentMetadata::default(),
        };
        
        document.set_content(content)?;
        Ok(document)
    }
    
    /// Get document content
    pub fn get_content(&self) -> &str {
        &self.content
    }
    
    /// Set document content and update internal structures
    pub fn set_content(&mut self, content: String) -> Result<()> {
        self.content = content;
        self.update_line_starts();
        self.detect_line_endings();
        self.update_metadata();
        
        // Reset position tracker
        self.position_tracker.reset();
        self.position_tracker.advance_str(&self.content);
        
        Ok(())
    }
    
    /// Get line content by line number (zero-indexed)
    pub fn get_line(&self, line_number: usize) -> Option<String> {
        if line_number >= self.line_starts.len() {
            return None;
        }
        
        let start = self.line_starts[line_number];
        let end = if line_number + 1 < self.line_starts.len() {
            self.line_starts[line_number + 1] - 1 // Exclude newline
        } else {
            self.content.len()
        };
        
        if start <= end && end <= self.content.len() {
            Some(self.content[start..end].to_string())
        } else {
            None
        }
    }
    
    /// Get character at position
    pub fn get_character_at(&self, position: Position) -> Option<char> {
        if let Ok(offset) = self.position_to_offset(position) {
            self.content.chars().nth(offset)
        } else {
            None
        }
    }
    
    /// Convert position to byte offset
    pub fn position_to_offset(&self, position: Position) -> Result<usize> {
        if position.line as usize >= self.line_starts.len() {
            return Err(LambdustError::runtime_error(
                format!("Line {} out of bounds", position.line)
            ));
        }
        
        let line_start = self.line_starts[position.line as usize];
        let line_content = self.get_line(position.line as usize)
            .ok_or_else(|| LambdustError::runtime_error("Invalid line"))?;
        
        // Convert UTF-16 character offset to UTF-8 byte offset
        let char_offset = position.character as usize;
        let mut utf8_offset = 0;
        let mut utf16_count = 0;
        
        for ch in line_content.chars() {
            if utf16_count >= char_offset {
                break;
            }
            utf16_count += ch.len_utf16();
            utf8_offset += ch.len_utf8();
        }
        
        Ok(line_start + utf8_offset)
    }
    
    /// Convert byte offset to position
    pub fn offset_to_position(&self, offset: usize) -> Result<Position> {
        if offset > self.content.len() {
            return Err(LambdustError::runtime_error("Offset out of bounds"));
        }
        
        // Find line containing offset
        let line = match self.line_starts.binary_search(&offset) {
            Ok(line) => line,
            Err(line) => line.saturating_sub(1),
        };
        
        if line >= self.line_starts.len() {
            return Err(LambdustError::runtime_error("Invalid line calculation"));
        }
        
        let line_start = self.line_starts[line];
        let byte_offset_in_line = offset - line_start;
        
        // Convert byte offset to UTF-16 character offset
        let line_content = self.get_line(line)
            .ok_or_else(|| LambdustError::runtime_error("Invalid line"))?;
        
        let mut character = 0u32;
        let mut byte_count = 0;
        
        for ch in line_content.chars() {
            if byte_count >= byte_offset_in_line {
                break;
            }
            character += ch.len_utf16() as u32;
            byte_count += ch.len_utf8();
        }
        
        Ok(Position::new(line as u32, character))
    }
    
    /// Get text in range
    pub fn get_text_range(&self, range: Range) -> Result<String> {
        use crate::lsp::position::PositionUtils;
        PositionUtils::extract_text(&self.content, range)
    }
    
    /// Get number of lines
    pub fn line_count(&self) -> usize {
        self.line_starts.len()
    }
    
    /// Update line starts cache
    fn update_line_starts(&mut self) {
        self.line_starts.clear();
        self.line_starts.push(0); // First line starts at 0
        
        for (i, ch) in self.content.char_indices() {
            if ch == '\n' {
                self.line_starts.push(i + ch.len_utf8());
            }
        }
    }
    
    /// Detect line ending style
    fn detect_line_endings(&mut self) {
        let mut has_crlf = false;
        let mut has_lf = false;
        let mut has_cr = false;
        
        let mut chars = self.content.chars().peekable();
        while let Some(ch) = chars.next() {
            match ch {
                '\r' => {
                    if chars.peek() == Some(&'\n') {
                        has_crlf = true;
                        chars.next(); // Skip the \n
                    } else {
                        has_cr = true;
                    }
                },
                '\n' => has_lf = true,
                _ => {}
            }
        }
        
        self.line_endings = match (has_crlf, has_lf, has_cr) {
            (true, false, false) => LineEndings::Windows,
            (false, true, false) => LineEndings::Unix,
            (false, false, true) => LineEndings::Mac,
            _ => LineEndings::Mixed,
        };
    }
    
    /// Update document metadata
    fn update_metadata(&mut self) {
        self.metadata.size = self.content.len();
        self.metadata.line_count = self.line_count();
        self.metadata.last_modified = Some(std::time::SystemTime::now());
        self.metadata.encoding = "utf-8".to_string();
        
        // Extract file path from URI if possible
        if let Ok(url) = Url::parse(&self.uri) {
            if let Ok(path) = url.to_file_path() {
                self.metadata.file_path = Some(path);
            }
        }
    }
    
    /// Get document metadata
    pub fn metadata(&self) -> &DocumentMetadata {
        &self.metadata
    }
    
    /// Get line endings style
    pub fn line_endings(&self) -> LineEndings {
        self.line_endings
    }
}

impl ChangeTracker {
    /// Create new change tracker
    pub fn new() -> Self {
        Self {
            changes: HashMap::new(),
            max_changes: 100,
        }
    }
    
    /// Add a change for a document
    pub fn add_change(&mut self, uri: &str, change: DocumentChange) {
        let changes = self.changes.entry(uri.to_string()).or_insert_with(Vec::new);
        changes.push(change);
        
        // Limit number of tracked changes
        if changes.len() > self.max_changes {
            changes.remove(0);
        }
    }
    
    /// Get recent changes for a document
    pub fn get_changes(&self, uri: &str) -> Option<&Vec<DocumentChange>> {
        self.changes.get(uri)
    }
    
    /// Clear changes for a document
    pub fn clear_changes(&mut self, uri: &str) {
        self.changes.remove(uri);
    }
    
    /// Clear all changes
    pub fn clear_all(&mut self) {
        self.changes.clear();
    }
}

impl Default for DocumentManager {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for ChangeTracker {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_document_creation() {
        let doc = Document::new(
            "file:///test.scm".to_string(),
            "scheme".to_string(),
            1,
            "(+ 1 2 3)".to_string(),
        ).unwrap();
        
        assert_eq!(doc.get_content(), "(+ 1 2 3)");
        assert_eq!(doc.line_count(), 1);
    }

    #[test]
    fn test_position_conversion() {
        let doc = Document::new(
            "file:///test.scm".to_string(),
            "scheme".to_string(),
            1,
            "hello\nworld".to_string(),
        ).unwrap();
        
        let pos = Position::new(1, 2);
        let offset = doc.position_to_offset(pos).unwrap();
        let back_to_pos = doc.offset_to_position(offset).unwrap();
        
        assert_eq!(pos, back_to_pos);
    }

    #[test]
    fn test_line_endings_detection() {
        let mut doc = Document::new(
            "file:///test.scm".to_string(),
            "scheme".to_string(),
            1,
            "line1\r\nline2\r\nline3".to_string(),
        ).unwrap();
        
        assert_eq!(doc.line_endings(), LineEndings::Windows);
    }

    #[test]
    fn test_document_manager() {
        let mut manager = DocumentManager::new();
        
        let text_doc = lsp_types::TextDocumentItem {
            uri: "file:///test.scm".parse().unwrap(),
            language_id: "scheme".to_string(),
            version: 1,
            text: "(+ 1 2)".to_string(),
        };
        
        tokio::runtime::Runtime::new().unwrap().block_on(async {
            manager.did_open(text_doc.clone()).await.unwrap();
            
            let doc = manager.get_document(&text_doc.uri);
            assert!(doc.is_some());
            assert_eq!(doc.unwrap().get_content(), "(+ 1 2)");
        });
    }
}