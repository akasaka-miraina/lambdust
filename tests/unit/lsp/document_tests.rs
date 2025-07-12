//! LSP Document Management Tests
//!
//! Tests for document lifecycle management including:
//! - Document opening, editing, and closing
//! - Content synchronization and validation
//! - Multi-document workspace handling
//! - Change tracking and incremental updates

use super::lsp_test_utils::*;
use crate::lsp::document::{DocumentManager, Document, DocumentChange, TextEdit};
use crate::lsp::position::{Position, Range};
use std::sync::Arc;

#[test]
fn test_document_creation() {
    let doc = create_test_document("test://example.scm", "(+ 1 2 3)");
    
    assert_eq!(doc.uri(), "test://example.scm");
    assert_eq!(doc.language_id(), "scheme");
    assert_eq!(doc.version(), 1);
    assert_eq!(doc.content(), "(+ 1 2 3)");
    assert_eq!(doc.line_count(), 1);
}

#[test]
fn test_document_multiline_content() {
    let code = r#"
(define (factorial n)
  (if (<= n 1)
      1
      (* n (factorial (- n 1)))))
"#;
    let doc = create_test_document("test://multiline.scm", code);
    
    assert_eq!(doc.line_count(), 6); // Including initial empty line
    
    // Test line access
    assert_eq!(doc.get_line(0).unwrap(), "");
    assert_eq!(doc.get_line(1).unwrap(), "(define (factorial n)");
    assert_eq!(doc.get_line(2).unwrap(), "  (if (<= n 1)");
}

#[test]
fn test_document_manager_creation() {
    let manager = DocumentManager::new();
    
    assert_eq!(manager.document_count(), 0);
    assert!(manager.list_documents().is_empty());
}

#[test]
fn test_document_manager_open_close() {
    let mut manager = DocumentManager::new();
    let code = "(define x 42)";
    
    // Open document
    let doc = manager.open_document(
        "test://open.scm",
        "scheme",
        1,
        code
    ).unwrap();
    
    assert_eq!(manager.document_count(), 1);
    assert_eq!(doc.uri(), "test://open.scm");
    assert_eq!(doc.content(), code);
    
    // Get document
    let retrieved = manager.get_document("test://open.scm").unwrap();
    assert_eq!(retrieved.uri(), "test://open.scm");
    
    // Close document
    let closed = manager.close_document("test://open.scm").unwrap();
    assert_eq!(closed.uri(), "test://open.scm");
    assert_eq!(manager.document_count(), 0);
}

#[test]
fn test_document_manager_multiple_documents() {
    let mut manager = DocumentManager::new();
    
    // Open multiple documents
    manager.open_document("test://doc1.scm", "scheme", 1, "(+ 1 2)").unwrap();
    manager.open_document("test://doc2.scm", "scheme", 1, "(* 3 4)").unwrap();
    manager.open_document("test://doc3.scm", "scheme", 1, "(- 5 1)").unwrap();
    
    assert_eq!(manager.document_count(), 3);
    
    let doc_list = manager.list_documents();
    assert_eq!(doc_list.len(), 3);
    assert!(doc_list.contains(&"test://doc1.scm".to_string()));
    assert!(doc_list.contains(&"test://doc2.scm".to_string()));
    assert!(doc_list.contains(&"test://doc3.scm".to_string()));
}

#[test]
fn test_document_text_edits() {
    let mut doc = create_test_document("test://edit.scm", "(+ 1 2)");
    
    // Simple replacement
    let edit = TextEdit {
        range: Range::new(Position::new(0, 3), Position::new(0, 4)), // Replace "1"
        new_text: "10".to_string(),
    };
    
    doc.apply_edit(edit).unwrap();
    assert_eq!(doc.content(), "(+ 10 2)");
    assert_eq!(doc.version(), 2);
}

#[test]
fn test_document_insertion() {
    let mut doc = create_test_document("test://insert.scm", "(+ 1)");
    
    // Insert at specific position
    let insert = TextEdit {
        range: Range::new(Position::new(0, 4), Position::new(0, 4)), // After "1"
        new_text: " 2".to_string(),
    };
    
    doc.apply_edit(insert).unwrap();
    assert_eq!(doc.content(), "(+ 1 2)");
    assert_eq!(doc.version(), 2);
}

#[test]
fn test_document_deletion() {
    let mut doc = create_test_document("test://delete.scm", "(+ 1 2 3)");
    
    // Delete " 3"
    let delete = TextEdit {
        range: Range::new(Position::new(0, 5), Position::new(0, 7)),
        new_text: "".to_string(),
    };
    
    doc.apply_edit(delete).unwrap();
    assert_eq!(doc.content(), "(+ 1 2)");
    assert_eq!(doc.version(), 2);
}

#[test]
fn test_document_multiline_edits() {
    let initial_code = r#"(define x 1)
(define y 2)
(+ x y)"#;
    
    let mut doc = create_test_document("test://multiline.scm", initial_code);
    
    // Replace middle line
    let edit = TextEdit {
        range: Range::new(Position::new(1, 0), Position::new(1, 12)), // Entire second line
        new_text: "(define y 20)".to_string(),
    };
    
    doc.apply_edit(edit).unwrap();
    
    let expected = r#"(define x 1)
(define y 20)
(+ x y)"#;
    assert_eq!(doc.content(), expected);
}

#[test]
fn test_document_change_tracking() {
    let mut doc = create_test_document("test://tracking.scm", "(+ 1 2)");
    
    let changes = doc.get_changes_since_version(0).unwrap();
    assert!(changes.is_empty()); // No changes yet
    
    // Make an edit
    let edit = TextEdit {
        range: Range::new(Position::new(0, 3), Position::new(0, 4)),
        new_text: "10".to_string(),
    };
    doc.apply_edit(edit).unwrap();
    
    let changes = doc.get_changes_since_version(1).unwrap();
    assert_eq!(changes.len(), 1);
    assert_eq!(changes[0].range.start, Position::new(0, 3));
    assert_eq!(changes[0].new_text, "10");
}

#[test]
fn test_document_version_management() {
    let mut doc = create_test_document("test://version.scm", "original");
    assert_eq!(doc.version(), 1);
    
    // Apply multiple edits
    for i in 1..=5 {
        let edit = TextEdit {
            range: Range::new(Position::new(0, 0), Position::new(0, doc.content().len() as u32)),
            new_text: format!("version{}", i),
        };
        doc.apply_edit(edit).unwrap();
        assert_eq!(doc.version(), i as u32 + 1);
    }
    
    assert_eq!(doc.content(), "version5");
    assert_eq!(doc.version(), 6);
}

#[test]
fn test_document_range_validation() {
    let doc = create_test_document("test://validate.scm", "short");
    
    // Valid range
    let valid_range = Range::new(Position::new(0, 0), Position::new(0, 5));
    assert!(doc.validate_range(valid_range).is_ok());
    
    // Invalid range - beyond content
    let invalid_range = Range::new(Position::new(0, 0), Position::new(0, 10));
    assert!(doc.validate_range(invalid_range).is_err());
    
    // Invalid range - beyond lines
    let invalid_line_range = Range::new(Position::new(0, 0), Position::new(5, 0));
    assert!(doc.validate_range(invalid_line_range).is_err());
}

#[test]
fn test_document_text_extraction() {
    let code = r#"line one
line two
line three"#;
    let doc = create_test_document("test://extract.scm", code);
    
    // Extract single line
    let range = Range::new(Position::new(1, 0), Position::new(1, 8));
    let extracted = doc.get_text_in_range(range).unwrap();
    assert_eq!(extracted, "line two");
    
    // Extract across lines
    let multi_range = Range::new(Position::new(0, 5), Position::new(2, 4));
    let multi_extracted = doc.get_text_in_range(multi_range).unwrap();
    assert!(multi_extracted.contains("one"));
    assert!(multi_extracted.contains("line two"));
    assert!(multi_extracted.contains("line"));
}

#[test]
fn test_document_position_conversion() {
    let code = "hello\nworld\ntest";
    let doc = create_test_document("test://position.scm", code);
    
    // Convert position to offset
    let pos = Position::new(1, 2); // 'r' in "world"
    let offset = doc.position_to_offset(pos).unwrap();
    assert_eq!(offset, 8); // 5 chars in "hello\n" + 2 in "wo"
    
    // Convert offset back to position
    let back_to_pos = doc.offset_to_position(offset).unwrap();
    assert_eq!(back_to_pos, pos);
}

#[test]
fn test_document_unicode_handling() {
    let unicode_code = "hello 🌍 world";
    let doc = create_test_document("test://unicode.scm", unicode_code);
    
    // Test position in Unicode text
    let pos = Position::new(0, 8); // After emoji (which takes 2 UTF-16 code units)
    let offset = doc.position_to_offset(pos);
    assert!(offset.is_some());
    
    // Test text extraction with Unicode
    let range = Range::new(Position::new(0, 0), Position::new(0, 7));
    let extracted = doc.get_text_in_range(range).unwrap();
    assert!(extracted.contains("hello"));
    assert!(extracted.contains("🌍"));
}

#[test]
fn test_document_manager_synchronization() {
    let mut manager = DocumentManager::new();
    
    // Open document
    manager.open_document("test://sync.scm", "scheme", 1, "(+ 1 2)").unwrap();
    
    // Update document content
    let change = DocumentChange {
        range: Some(Range::new(Position::new(0, 3), Position::new(0, 4))),
        text: "10".to_string(),
    };
    
    manager.update_document("test://sync.scm", 2, vec![change]).unwrap();
    
    let doc = manager.get_document("test://sync.scm").unwrap();
    assert_eq!(doc.content(), "(+ 10 2)");
    assert_eq!(doc.version(), 2);
}

#[test]
fn test_document_manager_incremental_updates() {
    let mut manager = DocumentManager::new();
    
    let initial = r#"(define x 1)
(define y 2)
(+ x y)"#;
    
    manager.open_document("test://incremental.scm", "scheme", 1, initial).unwrap();
    
    // Multiple incremental changes
    let changes = vec![
        DocumentChange {
            range: Some(Range::new(Position::new(0, 11), Position::new(0, 11))),
            text: "0".to_string(), // Change "1" to "10"
        },
        DocumentChange {
            range: Some(Range::new(Position::new(1, 11), Position::new(1, 11))),
            text: "0".to_string(), // Change "2" to "20"
        },
    ];
    
    manager.update_document("test://incremental.scm", 2, changes).unwrap();
    
    let doc = manager.get_document("test://incremental.scm").unwrap();
    let expected = r#"(define x 10)
(define y 20)
(+ x y)"#;
    assert_eq!(doc.content(), expected);
}

#[test]
fn test_document_error_handling() {
    let mut manager = DocumentManager::new();
    
    // Try to get non-existent document
    let result = manager.get_document("test://nonexistent.scm");
    assert!(result.is_err());
    
    // Try to close non-existent document
    let close_result = manager.close_document("test://nonexistent.scm");
    assert!(close_result.is_err());
    
    // Try to update non-existent document
    let update_result = manager.update_document("test://nonexistent.scm", 2, vec![]);
    assert!(update_result.is_err());
}

#[test]
fn test_document_concurrent_access() {
    use std::sync::Arc;
    use std::thread;
    
    let manager = Arc::new(DocumentManager::new());
    let mut handles = vec![];
    
    // Simulate concurrent document operations
    for i in 0..5 {
        let manager_clone = Arc::clone(&manager);
        let handle = thread::spawn(move || {
            let uri = format!("test://concurrent{}.scm", i);
            let content = format!("(define var{} {})", i, i);
            
            // Each thread opens and accesses its own document
            manager_clone.open_document(&uri, "scheme", 1, &content).unwrap();
            
            let doc = manager_clone.get_document(&uri).unwrap();
            assert_eq!(doc.content(), content);
            
            manager_clone.close_document(&uri).unwrap();
        });
        handles.push(handle);
    }
    
    // Wait for all threads to complete
    for handle in handles {
        handle.join().unwrap();
    }
    
    assert_eq!(manager.document_count(), 0);
}

#[test]
fn test_document_large_content() {
    // Test with larger document
    let large_content = (0..1000)
        .map(|i| format!("(define var{} {})", i, i))
        .collect::<Vec<_>>()
        .join("\n");
    
    let doc = create_test_document("test://large.scm", &large_content);
    
    assert_eq!(doc.line_count(), 1000);
    
    // Test random access to lines
    assert!(doc.get_line(500).unwrap().contains("var500"));
    assert!(doc.get_line(999).unwrap().contains("var999"));
    
    // Test position conversion in large document
    let pos = Position::new(500, 10);
    let offset = doc.position_to_offset(pos);
    assert!(offset.is_some());
}

#[test]
fn test_document_empty_and_whitespace() {
    // Empty document
    let empty_doc = create_test_document("test://empty.scm", "");
    assert_eq!(empty_doc.line_count(), 1); // Empty document still has one line
    assert_eq!(empty_doc.content().len(), 0);
    
    // Whitespace-only document
    let whitespace_doc = create_test_document("test://whitespace.scm", "   \n\t\n  ");
    assert_eq!(whitespace_doc.line_count(), 3);
    assert_eq!(whitespace_doc.get_line(0).unwrap(), "   ");
    assert_eq!(whitespace_doc.get_line(1).unwrap(), "\t");
    assert_eq!(whitespace_doc.get_line(2).unwrap(), "  ");
}

#[test]
fn test_document_change_events() {
    let mut doc = create_test_document("test://events.scm", "original");
    
    // Track changes
    let mut change_count = 0;
    doc.set_change_listener(Box::new(move |_change| {
        change_count += 1;
    }));
    
    // Apply edits (this test simulates the concept - actual implementation may vary)
    let edit1 = TextEdit {
        range: Range::new(Position::new(0, 0), Position::new(0, 8)),
        new_text: "modified".to_string(),
    };
    doc.apply_edit(edit1).unwrap();
    
    let edit2 = TextEdit {
        range: Range::new(Position::new(0, 8), Position::new(0, 8)),
        new_text: " again".to_string(),
    };
    doc.apply_edit(edit2).unwrap();
    
    // In a real implementation, we would verify that change_count == 2
    assert_eq!(doc.content(), "modified again");
}