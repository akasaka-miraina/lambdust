//! Unit tests for SRFI 130: Cursor-based String Library

use lambdust::lexer::SchemeNumber;
use lambdust::srfi::srfi_130::{Srfi130, StringCursor};
use lambdust::srfi::SrfiModule;
use lambdust::value::{Procedure, Value};
use std::rc::Rc;

#[test]
fn test_string_cursor_creation() {
    let cursor = StringCursor::new("hello".to_string());
    assert_eq!(cursor.string(), "hello");
    assert_eq!(cursor.position(), 0);
    assert_eq!(cursor.start(), 0);
    assert_eq!(cursor.end(), 5);
    assert!(cursor.at_start());
    assert!(!cursor.at_end());
}

#[test]
fn test_string_cursor_navigation() {
    let mut cursor = StringCursor::new("hello".to_string());

    // Test initial position
    assert_eq!(cursor.current_char().unwrap(), 'h');

    // Advance through string
    cursor.advance().unwrap();
    assert_eq!(cursor.position(), 1);
    assert_eq!(cursor.current_char().unwrap(), 'e');

    cursor.advance().unwrap();
    assert_eq!(cursor.position(), 2);
    assert_eq!(cursor.current_char().unwrap(), 'l');

    // Retreat
    cursor.retreat().unwrap();
    assert_eq!(cursor.position(), 1);
    assert_eq!(cursor.current_char().unwrap(), 'e');

    // Test boundaries - create new cursors at boundaries
    let mut start_cursor = StringCursor::new("hello".to_string());
    assert!(start_cursor.retreat().is_err()); // Cannot retreat past start

    let mut end_cursor = StringCursor::new("hello".to_string());
    // Move to end
    while !end_cursor.at_end() {
        end_cursor.advance().unwrap();
    }
    assert!(end_cursor.advance().is_err()); // Cannot advance past end
}

#[test]
fn test_string_cursor_bounds() {
    let cursor = StringCursor::with_bounds("hello world".to_string(), 2, 7).unwrap();
    assert_eq!(cursor.position(), 2);
    assert_eq!(cursor.start(), 2);
    assert_eq!(cursor.end(), 7);
    assert_eq!(cursor.rest(), "llo w");
    assert_eq!(cursor.prefix(), "");

    // Test invalid bounds
    assert!(StringCursor::with_bounds("hello".to_string(), 3, 2).is_err()); // start > end
    assert!(StringCursor::with_bounds("hello".to_string(), 0, 10).is_err()); // end > string length
}

#[test]
fn test_unicode_support() {
    let mut cursor = StringCursor::new("こんにちは".to_string());

    // Each Japanese character is 3 bytes in UTF-8
    assert_eq!(cursor.string().len(), 15); // 5 chars * 3 bytes each

    // Test character navigation
    assert_eq!(cursor.current_char().unwrap(), 'こ');

    cursor.advance().unwrap();
    assert_eq!(cursor.position(), 3); // First character boundary
    assert_eq!(cursor.current_char().unwrap(), 'ん');

    cursor.advance().unwrap();
    assert_eq!(cursor.position(), 6); // Second character boundary
    assert_eq!(cursor.current_char().unwrap(), 'に');

    // Test retreat
    cursor.retreat().unwrap();
    assert_eq!(cursor.position(), 3);
    assert_eq!(cursor.current_char().unwrap(), 'ん');
}

#[test]
fn test_string_cursor_procedures() {
    let srfi = Srfi130;
    let exports = srfi.exports();

    // Test string-cursor-start
    let start_proc = exports.get("string-cursor-start").unwrap();
    if let Value::Procedure(Procedure::Builtin { func, .. }) = start_proc {
        let result = func(&[Value::String("hello".to_string())]).unwrap();
        assert!(matches!(result, Value::StringCursor(_)));

        if let Value::StringCursor(cursor) = result {
            assert_eq!(cursor.position(), 0);
            assert_eq!(cursor.string(), "hello");
        }
    }

    // Test string-cursor-end
    let end_proc = exports.get("string-cursor-end").unwrap();
    if let Value::Procedure(Procedure::Builtin { func, .. }) = end_proc {
        let result = func(&[Value::String("hello".to_string())]).unwrap();
        assert!(matches!(result, Value::StringCursor(_)));

        if let Value::StringCursor(cursor) = result {
            assert_eq!(cursor.position(), 5); // At end
            assert_eq!(cursor.string(), "hello");
        }
    }
}

#[test]
fn test_string_cursor_predicate() {
    let srfi = Srfi130;
    let exports = srfi.exports();

    let pred = exports.get("string-cursor?").unwrap();
    if let Value::Procedure(Procedure::Builtin { func, .. }) = pred {
        // Test with string cursor
        let cursor = Value::StringCursor(Rc::new(StringCursor::new("test".to_string())));
        let result = func(&[cursor]).unwrap();
        assert_eq!(result, Value::Boolean(true));

        // Test with non-cursor
        let result = func(&[Value::String("test".to_string())]).unwrap();
        assert_eq!(result, Value::Boolean(false));
    }
}

#[test]
fn test_string_cursor_navigation_procedures() {
    let srfi = Srfi130;
    let exports = srfi.exports();

    let next_proc = exports.get("string-cursor-next").unwrap();
    let prev_proc = exports.get("string-cursor-prev").unwrap();

    if let (
        Value::Procedure(Procedure::Builtin {
            func: next_func, ..
        }),
        Value::Procedure(Procedure::Builtin {
            func: prev_func, ..
        }),
    ) = (next_proc, prev_proc)
    {
        let cursor = Value::StringCursor(Rc::new(StringCursor::new("hello".to_string())));

        // Test next
        let next_cursor = next_func(&[cursor.clone()]).unwrap();
        if let Value::StringCursor(ref next_cursor_rc) = next_cursor {
            assert_eq!(next_cursor_rc.position(), 1);
        }

        // Test prev (from position 1)
        let prev_cursor = prev_func(&[next_cursor]).unwrap();
        if let Value::StringCursor(prev_cursor_rc) = prev_cursor {
            assert_eq!(prev_cursor_rc.position(), 0);
        }
    }
}

#[test]
fn test_string_cursor_comparison() {
    let srfi = Srfi130;
    let exports = srfi.exports();

    let eq_proc = exports.get("string-cursor=?").unwrap();
    let lt_proc = exports.get("string-cursor<?").unwrap();

    if let (
        Value::Procedure(Procedure::Builtin { func: eq_func, .. }),
        Value::Procedure(Procedure::Builtin { func: lt_func, .. }),
    ) = (eq_proc, lt_proc)
    {
        let cursor1 = StringCursor::new("hello".to_string());
        let cursor2 = StringCursor::new("hello".to_string());
        let mut cursor3 = StringCursor::new("hello".to_string());
        cursor3.advance().unwrap(); // Move to position 1

        let cursor1_val = Value::StringCursor(Rc::new(cursor1));
        let cursor2_val = Value::StringCursor(Rc::new(cursor2));
        let cursor3_val = Value::StringCursor(Rc::new(cursor3));

        // Test equality
        let result = eq_func(&[cursor1_val.clone(), cursor2_val]).unwrap();
        assert_eq!(result, Value::Boolean(true));

        let result = eq_func(&[cursor1_val.clone(), cursor3_val.clone()]).unwrap();
        assert_eq!(result, Value::Boolean(false));

        // Test less than
        let result = lt_func(&[cursor1_val, cursor3_val]).unwrap();
        assert_eq!(result, Value::Boolean(true));
    }
}

#[test]
fn test_string_cursor_ref() {
    let srfi = Srfi130;
    let exports = srfi.exports();

    let ref_proc = exports.get("string-cursor-ref").unwrap();
    if let Value::Procedure(Procedure::Builtin { func, .. }) = ref_proc {
        let cursor = StringCursor::new("hello".to_string());
        let cursor_val = Value::StringCursor(Rc::new(cursor));

        let result = func(&[cursor_val]).unwrap();
        assert_eq!(result, Value::Character('h'));
    }
}

#[test]
fn test_substring_cursors() {
    let srfi = Srfi130;
    let exports = srfi.exports();

    let substr_proc = exports.get("substring/cursors").unwrap();
    if let Value::Procedure(Procedure::Builtin { func, .. }) = substr_proc {
        // Create cursors at specific positions using with_bounds
        let mut start_cursor = StringCursor::new("hello world".to_string());
        // Advance to position 2 manually
        start_cursor.advance().unwrap(); // 1
        start_cursor.advance().unwrap(); // 2

        let mut end_cursor = StringCursor::new("hello world".to_string());
        // Advance to position 7 manually
        for _ in 0..7 {
            end_cursor.advance().unwrap();
        }

        let start_val = Value::StringCursor(Rc::new(start_cursor));
        let end_val = Value::StringCursor(Rc::new(end_cursor));

        let result = func(&[start_val, end_val]).unwrap();
        assert_eq!(result, Value::String("llo w".to_string()));
    }
}

#[test]
fn test_string_search_cursors() {
    let srfi = Srfi130;
    let exports = srfi.exports();

    // Test string-index-cursor
    let index_proc = exports.get("string-index-cursor").unwrap();
    if let Value::Procedure(Procedure::Builtin { func, .. }) = index_proc {
        let result = func(&[Value::String("hello".to_string()), Value::Character('l')]).unwrap();

        if let Value::StringCursor(cursor) = result {
            assert_eq!(cursor.position(), 2); // First 'l' at position 2
        } else {
            panic!("Expected string cursor");
        }

        // Test character not found
        let result = func(&[Value::String("hello".to_string()), Value::Character('x')]).unwrap();
        assert_eq!(result, Value::Boolean(false));
    }

    // Test string-contains-cursor
    let contains_proc = exports.get("string-contains-cursor").unwrap();
    if let Value::Procedure(Procedure::Builtin { func, .. }) = contains_proc {
        let result = func(&[
            Value::String("hello world".to_string()),
            Value::String("world".to_string()),
        ])
        .unwrap();

        if let Value::StringCursor(cursor) = result {
            assert_eq!(cursor.position(), 6); // "world" starts at position 6
        } else {
            panic!("Expected string cursor");
        }

        // Test substring not found
        let result = func(&[
            Value::String("hello".to_string()),
            Value::String("xyz".to_string()),
        ])
        .unwrap();
        assert_eq!(result, Value::Boolean(false));
    }
}

#[test]
fn test_string_operations_with_cursors() {
    let srfi = Srfi130;
    let exports = srfi.exports();

    // Test string-take-cursor
    let take_proc = exports.get("string-take-cursor").unwrap();
    if let Value::Procedure(Procedure::Builtin { func, .. }) = take_proc {
        let result = func(&[
            Value::String("hello".to_string()),
            Value::Number(SchemeNumber::Integer(3)),
        ])
        .unwrap();
        assert_eq!(result, Value::String("hel".to_string()));
    }

    // Test string-drop-cursor
    let drop_proc = exports.get("string-drop-cursor").unwrap();
    if let Value::Procedure(Procedure::Builtin { func, .. }) = drop_proc {
        let result = func(&[
            Value::String("hello".to_string()),
            Value::Number(SchemeNumber::Integer(2)),
        ])
        .unwrap();
        assert_eq!(result, Value::String("llo".to_string()));
    }
}

#[test]
fn test_srfi_130_metadata() {
    let srfi = Srfi130;
    assert_eq!(srfi.srfi_id(), 130);
    assert_eq!(srfi.name(), "Cursor-based String Library");
    assert!(srfi.parts().is_empty());
}

#[test]
fn test_srfi_130_exports() {
    let srfi = Srfi130;
    let exports = srfi.exports();

    // Check key procedures are exported
    let expected_exports = [
        "string-cursor-start",
        "string-cursor-end",
        "string-cursor?",
        "string-cursor-next",
        "string-cursor-prev",
        "string-cursor=?",
        "string-cursor<?",
        "string-cursor-ref",
        "substring/cursors",
        "string-take-cursor",
        "string-drop-cursor",
        "string-index-cursor",
        "string-contains-cursor",
        "string-length/cursors",
    ];

    for export in &expected_exports {
        assert!(exports.contains_key(*export), "Missing export: {}", export);
    }
}

#[test]
fn test_exports_for_parts() {
    let srfi = Srfi130;
    let all_exports = srfi.exports();
    let parts_exports = srfi.exports_for_parts(&[]).unwrap();

    // Should return all exports since SRFI 130 has no parts
    assert_eq!(all_exports.len(), parts_exports.len());
}
