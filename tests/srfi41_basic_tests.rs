//! Basic tests for SRFI-41 Streams implementation.
//!
//! This test suite validates the core functionality of the SRFI-41 streams
//! implementation following the architectural specifications.

use lambdust::eval::stream::{StreamNode, StreamError};
use lambdust::eval::value::{Value, Promise};
use lambdust::stdlib::srfi41_streams::SRFI41Streams;
use lambdust::eval::environment::ThreadSafeEnvironment;
use lambdust::ast::literal::Literal;

use std::sync::Arc;
use std::cell::RefCell;
use std::rc::Rc;

/// Helper function to create integer values
fn make_integer(n: i64) -> Value {
    Value::Literal(Literal::ExactInteger(n))
}

/// Helper function to create boolean values
fn make_boolean(b: bool) -> Value {
    Value::Literal(Literal::Boolean(b))
}

#[test]
fn test_stream_node_creation() {
    // Test empty stream creation
    let empty_stream = StreamNode::empty();
    assert!(empty_stream.is_empty());
    assert_eq!(empty_stream.generation(), 0);
}

#[test]
fn test_stream_node_evaluated_construction() {
    let empty_tail = StreamNode::empty();
    let head_value = make_integer(42);
    
    // Test evaluated stream construction
    let stream = StreamNode::evaluated(head_value.clone(), empty_tail);
    assert!(!stream.is_empty());
    
    // Test head access
    let retrieved_head = stream.head();
    assert!(retrieved_head.is_ok());
    assert_eq!(*retrieved_head.unwrap(), head_value);
}

#[test]
fn test_stream_cons_construction() {
    let head_value = make_integer(42);
    // For testing, we'll create a simple promise that returns empty stream
    let tail_promise = Rc::new(RefCell::new(Promise::Delayed {
        thunk: make_integer(0), // Placeholder thunk
    }));
    
    let stream = StreamNode::cons(head_value.clone(), tail_promise);
    assert!(!stream.is_empty());
    
    // Test head access
    let retrieved_head = stream.head();
    assert!(retrieved_head.is_ok());
    assert_eq!(*retrieved_head.unwrap(), head_value);
}

#[test]
fn test_stream_predicates() {
    // Test stream? predicate
    let empty_stream = Value::Stream(StreamNode::empty());
    let result = SRFI41Streams::stream_predicate(&[empty_stream]);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), make_boolean(true));
    
    // Test non-stream value
    let result = SRFI41Streams::stream_predicate(&[make_integer(42)]);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), make_boolean(false));
}

#[test]
fn test_stream_null_predicate() {
    // Test empty stream
    let empty_stream = Value::Stream(StreamNode::empty());
    let result = SRFI41Streams::stream_null_predicate(&[empty_stream]);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), make_boolean(true));
    
    // Test non-empty stream
    let head = make_integer(42);
    let tail = StreamNode::empty();
    let non_empty_stream = Value::Stream(StreamNode::evaluated(head, tail));
    let result = SRFI41Streams::stream_null_predicate(&[non_empty_stream]);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), make_boolean(false));
}

#[test]
fn test_stream_from_args() {
    // Test empty stream creation
    let result = SRFI41Streams::stream_from_args(&[]);
    assert!(result.is_ok());
    
    if let Ok(Value::Stream(stream)) = result {
        assert!(stream.is_empty());
    } else {
        panic!("Expected stream value");
    }
    
    // Test single element stream
    let result = SRFI41Streams::stream_from_args(&[make_integer(42)]);
    assert!(result.is_ok());
    
    if let Ok(Value::Stream(stream)) = result {
        assert!(!stream.is_empty());
        let head = stream.head();
        assert!(head.is_ok());
        assert_eq!(*head.unwrap(), make_integer(42));
    } else {
        panic!("Expected stream value");
    }
}

#[test]
fn test_list_to_stream_conversion() {
    // Test empty list conversion
    let result = SRFI41Streams::list_to_stream(&[Value::Nil]);
    assert!(result.is_ok());
    
    if let Ok(Value::Stream(stream)) = result {
        assert!(stream.is_empty());
    } else {
        panic!("Expected stream value");
    }
    
    // Test single element list conversion
    let list = Value::pair(make_integer(42), Value::Nil);
    let result = SRFI41Streams::list_to_stream(&[list]);
    assert!(result.is_ok());
    
    if let Ok(Value::Stream(stream)) = result {
        assert!(!stream.is_empty());
        let head = stream.head();
        assert!(head.is_ok());
        assert_eq!(*head.unwrap(), make_integer(42));
    } else {
        panic!("Expected stream value");
    }
}

#[test]
fn test_stream_to_list_conversion() {
    // Test empty stream conversion
    let empty_stream = StreamNode::empty();
    let result = SRFI41Streams::stream_to_list(&[Value::Stream(empty_stream)]);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Value::Nil);
    
    // Test single element stream conversion
    let head = make_integer(42);
    let tail = StreamNode::empty();
    let stream = StreamNode::evaluated(head.clone(), tail);
    let result = SRFI41Streams::stream_to_list(&[Value::Stream(stream)]);
    assert!(result.is_ok());
    
    let expected_list = Value::pair(head, Value::Nil);
    assert_eq!(result.unwrap(), expected_list);
}

#[test]
fn test_environment_binding() {
    let env = Arc::new(ThreadSafeEnvironment::new(None, 0));
    SRFI41Streams::bind_all_operations(&env);
    
    // Test that all core procedures are bound
    assert!(env.lookup("stream-null".to_string()).is_some());
    assert!(env.lookup("stream-cons".to_string()).is_some());
    assert!(env.lookup("stream".to_string()).is_some());
    assert!(env.lookup("stream-car".to_string()).is_some());
    assert!(env.lookup("stream-cdr".to_string()).is_some());
    assert!(env.lookup("stream?".to_string()).is_some());
    assert!(env.lookup("stream-null?".to_string()).is_some());
    assert!(env.lookup("list->stream".to_string()).is_some());
    assert!(env.lookup("stream->list".to_string()).is_some());
    assert!(env.lookup("stream-take".to_string()).is_some());
    assert!(env.lookup("stream-drop".to_string()).is_some());
    assert!(env.lookup("stream-append".to_string()).is_some());
    assert!(env.lookup("stream-map".to_string()).is_some());
}

#[test]
fn test_stream_error_handling() {
    // Test error handling for invalid arguments
    let result = SRFI41Streams::stream_predicate(&[]);
    assert!(result.is_err());
    
    let result = SRFI41Streams::stream_null_predicate(&[make_integer(42)]);
    assert!(result.is_err());
    
    let result = SRFI41Streams::list_to_stream(&[make_integer(42)]);
    assert!(result.is_err());
}

#[test]
fn test_stream_node_memory_layout() {
    use std::mem;
    
    // Verify that StreamNode has reasonable size for cache efficiency
    let size = mem::size_of::<lambdust::eval::stream::StreamNode>();
    assert!(size <= 64, "StreamNode size {} exceeds reasonable cache line size", size);
    
    // Test that ArenaMetadata has expected size
    let metadata_size = mem::size_of::<lambdust::eval::stream::ArenaMetadata>();
    assert!(metadata_size <= 16, "ArenaMetadata size {} is too large", metadata_size);
}