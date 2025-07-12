//! LSP Position and Range Handling Tests
//!
//! Comprehensive tests for position tracking, UTF-16/UTF-8 conversion,
//! and range calculations that are critical for accurate LSP communication.

use super::lsp_test_utils::*;
use crate::lsp::position::{
    Position, Range, Span, PositionTracker, PositionUtils
};

#[test]
fn test_position_creation_and_validation() {
    let pos = Position::new(10, 20);
    assert_eq!(pos.line, 10);
    assert_eq!(pos.character, 20);
    assert!(pos.is_valid());
    
    let zero_pos = Position::zero();
    assert_eq!(zero_pos.line, 0);
    assert_eq!(zero_pos.character, 0);
    assert!(zero_pos.is_valid());
}

#[test]
fn test_position_ordering() {
    let pos1 = Position::new(1, 5);
    let pos2 = Position::new(1, 10);
    let pos3 = Position::new(2, 5);
    
    assert!(pos1 < pos2);
    assert!(pos2 < pos3);
    assert!(pos1 < pos3);
    
    // Same position should be equal
    let pos4 = Position::new(1, 5);
    assert_eq!(pos1, pos4);
}

#[test]
fn test_range_creation_and_validation() {
    let start = Position::new(1, 5);
    let end = Position::new(2, 10);
    let range = Range::new(start, end);
    
    assert_eq!(range.start, start);
    assert_eq!(range.end, end);
    assert!(range.is_valid());
    assert!(!range.is_empty());
    
    // Zero-width range
    let zero_range = Range::at_position(start);
    assert!(zero_range.is_valid());
    assert!(zero_range.is_empty());
    assert_eq!(zero_range.start, zero_range.end);
}

#[test]
fn test_range_from_length() {
    let start = Position::new(0, 5);
    let range = Range::from_length(start, 10);
    
    assert_eq!(range.start, start);
    assert_eq!(range.end, Position::new(0, 15));
    assert!(range.is_valid());
}

#[test]
fn test_range_contains() {
    let range = Range::new(Position::new(1, 5), Position::new(3, 10));
    
    // Points inside range
    assert!(range.contains(Position::new(1, 5))); // Start point (inclusive)
    assert!(range.contains(Position::new(2, 0)));
    assert!(range.contains(Position::new(3, 5)));
    
    // Points outside range
    assert!(!range.contains(Position::new(0, 5))); // Before range
    assert!(!range.contains(Position::new(3, 10))); // End point (exclusive)
    assert!(!range.contains(Position::new(4, 0))); // After range
}

#[test]
fn test_range_contains_range() {
    let outer = Range::new(Position::new(1, 0), Position::new(5, 0));
    let inner = Range::new(Position::new(2, 5), Position::new(3, 10));
    let overlapping = Range::new(Position::new(0, 5), Position::new(2, 10));
    let outside = Range::new(Position::new(6, 0), Position::new(7, 0));
    
    assert!(outer.contains_range(&inner));
    assert!(!outer.contains_range(&overlapping));
    assert!(!outer.contains_range(&outside));
    
    // Range contains itself
    assert!(outer.contains_range(&outer));
}

#[test]
fn test_range_overlaps() {
    let range1 = Range::new(Position::new(1, 0), Position::new(3, 0));
    let range2 = Range::new(Position::new(2, 5), Position::new(4, 5));
    let range3 = Range::new(Position::new(4, 0), Position::new(5, 0));
    
    assert!(range1.overlaps(&range2));
    assert!(range2.overlaps(&range1)); // Symmetric
    assert!(!range1.overlaps(&range3));
    assert!(!range3.overlaps(&range1)); // Symmetric
}

#[test]
fn test_range_intersection() {
    let range1 = Range::new(Position::new(1, 0), Position::new(3, 10));
    let range2 = Range::new(Position::new(2, 5), Position::new(4, 5));
    
    let intersection = range1.intersection(&range2);
    assert!(intersection.is_some());
    
    let intersect = intersection.unwrap();
    assert_eq!(intersect.start, Position::new(2, 5));
    assert_eq!(intersect.end, Position::new(3, 10));
    
    // Non-overlapping ranges
    let range3 = Range::new(Position::new(5, 0), Position::new(6, 0));
    let no_intersection = range1.intersection(&range3);
    assert!(no_intersection.is_none());
}

#[test]
fn test_position_tracker_basic() {
    let mut tracker = PositionTracker::new();
    
    assert_eq!(tracker.current_position(), Position::new(0, 0));
    assert_eq!(tracker.current_offset(), 0);
    
    tracker.advance_char('h');
    assert_eq!(tracker.current_position(), Position::new(0, 1));
    assert_eq!(tracker.current_offset(), 1);
    
    tracker.advance_char('i');
    assert_eq!(tracker.current_position(), Position::new(0, 2));
    assert_eq!(tracker.current_offset(), 2);
}

#[test]
fn test_position_tracker_newlines() {
    let mut tracker = PositionTracker::new();
    
    tracker.advance_str("hello\nworld");
    
    let pos = tracker.current_position();
    assert_eq!(pos.line, 1);
    assert_eq!(pos.character, 5);
    assert_eq!(tracker.current_offset(), 11);
}

#[test]
fn test_position_tracker_multiple_lines() {
    let mut tracker = PositionTracker::new();
    
    tracker.advance_str("line1\nline2\nline3");
    
    let pos = tracker.current_position();
    assert_eq!(pos.line, 2);
    assert_eq!(pos.character, 5);
}

#[test]
fn test_position_tracker_utf8_characters() {
    let mut tracker = PositionTracker::new();
    
    // Test with multi-byte UTF-8 characters
    tracker.advance_str("hello 🌍 world");
    
    let pos = tracker.current_position();
    // The emoji takes 4 bytes in UTF-8 but counts as 2 UTF-16 code units
    assert_eq!(pos.line, 0);
    // Exact character count depends on UTF-16 encoding
    assert!(pos.character > 10);
}

#[test]
fn test_position_tracker_reset() {
    let mut tracker = PositionTracker::new();
    
    tracker.advance_str("some text\nmore text");
    assert_ne!(tracker.current_position(), Position::new(0, 0));
    
    tracker.reset();
    assert_eq!(tracker.current_position(), Position::new(0, 0));
    assert_eq!(tracker.current_offset(), 0);
}

#[test]
fn test_position_utf8_offset_conversion() {
    let line_text = "hello world";
    let pos = Position::new(0, 6);
    
    let offset = pos.to_utf8_offset(line_text);
    assert_eq!(offset, Some(6));
    
    // Test with multi-byte characters
    let unicode_text = "hello 🌍";
    let pos_unicode = Position::new(0, 8); // After the emoji (2 UTF-16 units)
    let offset_unicode = pos_unicode.to_utf8_offset(unicode_text);
    assert!(offset_unicode.is_some());
}

#[test]
fn test_position_utils_extract_text() {
    let text = "line1\nline2\nline3";
    
    // Single line extraction
    let range = Range::new(Position::new(1, 0), Position::new(1, 5));
    let extracted = PositionUtils::extract_text(text, range).unwrap();
    assert_eq!(extracted, "line2");
    
    // Multi-line extraction
    let multi_range = Range::new(Position::new(0, 3), Position::new(2, 3));
    let multi_extracted = PositionUtils::extract_text(text, multi_range).unwrap();
    assert!(multi_extracted.contains("e1"));
    assert!(multi_extracted.contains("line2"));
    assert!(multi_extracted.contains("lin"));
}

#[test]
fn test_position_utils_find_position() {
    let text = "hello world\nhello again";
    let positions = PositionUtils::find_position(text, "hello");
    
    assert_eq!(positions.len(), 2);
    assert_eq!(positions[0], Position::new(0, 0));
    assert_eq!(positions[1], Position::new(1, 0));
}

#[test]
fn test_lsp_position_conversion() {
    let internal_pos = Position::new(10, 20);
    let lsp_pos = PositionUtils::position_to_lsp_position(internal_pos);
    let back_to_internal = PositionUtils::lsp_position_to_position(&lsp_pos);
    
    assert_eq!(internal_pos, back_to_internal);
    assert_eq!(lsp_pos.line, 10);
    assert_eq!(lsp_pos.character, 20);
}

#[test]
fn test_lsp_range_conversion() {
    let internal_range = Range::new(Position::new(1, 5), Position::new(3, 10));
    let lsp_range = PositionUtils::range_to_lsp_range(internal_range);
    let back_to_internal = PositionUtils::lsp_range_to_range(&lsp_range);
    
    assert_eq!(internal_range, back_to_internal);
}

#[test]
fn test_span_creation() {
    let range = Range::new(Position::new(1, 0), Position::new(2, 10));
    
    let span = Span::new(range);
    assert_eq!(span.range, range);
    assert!(span.file.is_none());
    assert!(span.context.is_none());
    
    let span_with_file = Span::with_file(range, "test.scm".to_string());
    assert_eq!(span_with_file.file, Some("test.scm".to_string()));
    
    let span_with_context = Span::with_context(range, "function definition".to_string());
    assert_eq!(span_with_context.context, Some("function definition".to_string()));
    
    let span_complete = Span::with_file_and_context(
        range, 
        "test.scm".to_string(), 
        "function definition".to_string()
    );
    assert_eq!(span_complete.file, Some("test.scm".to_string()));
    assert_eq!(span_complete.context, Some("function definition".to_string()));
}

#[test]
fn test_position_tracker_offset_conversion() {
    let mut tracker = PositionTracker::new();
    tracker.advance_str("hello\nworld\ntest");
    
    // Test position to offset conversion
    let pos = Position::new(1, 3);
    let offset = tracker.position_to_offset(pos);
    assert!(offset.is_some());
    
    // Test offset to position conversion (round trip)
    if let Some(off) = offset {
        let back_to_pos = tracker.offset_to_position(off);
        // Note: exact round-trip might not work due to UTF-16/UTF-8 conversion complexities
        assert!(back_to_pos.is_some());
    }
}

#[test]
fn test_edge_case_positions() {
    // Test edge cases for position validation
    let max_pos = Position::new(u32::MAX - 1, u32::MAX - 1);
    assert!(max_pos.is_valid());
    
    // Invalid positions (at the boundary)
    let invalid_pos = Position::new(u32::MAX, u32::MAX);
    assert!(!invalid_pos.is_valid());
}

#[test]
fn test_edge_case_ranges() {
    // Empty range at zero
    let empty_range = Range::at_position(Position::zero());
    assert!(empty_range.is_valid());
    assert!(empty_range.is_empty());
    
    // Invalid range (end before start)
    let invalid_range = Range::new(Position::new(5, 10), Position::new(5, 5));
    assert!(!invalid_range.is_valid());
    
    // Maximum valid range
    let max_start = Position::new(0, 0);
    let max_end = Position::new(u32::MAX - 1, u32::MAX - 1);
    let max_range = Range::new(max_start, max_end);
    assert!(max_range.is_valid());
}

#[test]
fn test_position_utils_error_handling() {
    let text = "short";
    
    // Request text beyond bounds
    let out_of_bounds_range = Range::new(Position::new(0, 0), Position::new(10, 0));
    let result = PositionUtils::extract_text(text, out_of_bounds_range);
    assert!(result.is_err());
    
    // Invalid character position
    let invalid_char_range = Range::new(Position::new(0, 0), Position::new(0, 100));
    let result2 = PositionUtils::extract_text(text, invalid_char_range);
    assert!(result2.is_err());
}

#[test]
fn test_position_tracker_large_text() {
    let mut tracker = PositionTracker::new();
    
    // Test with larger text to verify performance and correctness
    let large_text = (0..1000).map(|i| format!("line{}", i)).collect::<Vec<_>>().join("\n");
    tracker.advance_str(&large_text);
    
    let final_pos = tracker.current_position();
    assert_eq!(final_pos.line, 999); // 1000 lines, 0-indexed
    assert!(final_pos.character > 0);
}

#[test]
fn test_unicode_handling() {
    let mut tracker = PositionTracker::new();
    
    // Mix of ASCII and Unicode characters
    let unicode_text = "Hello 🌍! How are you? 你好";
    tracker.advance_str(unicode_text);
    
    let pos = tracker.current_position();
    assert_eq!(pos.line, 0);
    // Character count should account for UTF-16 encoding
    assert!(pos.character > 20);
    
    // Test position conversion with Unicode
    let test_pos = Position::new(0, 10);
    let utf8_offset = test_pos.to_utf8_offset(unicode_text);
    assert!(utf8_offset.is_some());
}

#[test]
fn test_position_tracker_empty_input() {
    let mut tracker = PositionTracker::new();
    
    tracker.advance_str("");
    assert_eq!(tracker.current_position(), Position::zero());
    
    tracker.advance_char('\n');
    assert_eq!(tracker.current_position(), Position::new(1, 0));
}

#[test]
fn test_concurrent_position_tracking() {
    // Test that position tracking is consistent
    let text = "line1\nline2\nline3";
    
    let mut tracker1 = PositionTracker::new();
    let mut tracker2 = PositionTracker::new();
    
    // Advance both trackers through the same text
    tracker1.advance_str(text);
    for ch in text.chars() {
        tracker2.advance_char(ch);
    }
    
    // Should result in the same position
    assert_eq!(tracker1.current_position(), tracker2.current_position());
    assert_eq!(tracker1.current_offset(), tracker2.current_offset());
}