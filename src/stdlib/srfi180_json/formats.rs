//! JSON format support for SRFI-180
//!
//! This module implements support for:
//! - JSON Lines format (http://jsonlines.org/)
//! - JSON Text Sequences (RFC 7464)
//! - Streaming parsers for both formats

use crate::eval::value::Value;
use crate::stdlib::srfi180_json::error::{JsonError, JsonOptions, JsonResult};
use crate::stdlib::srfi180_json::parser::{parse_json_with_options, JsonParser};
use crate::stdlib::srfi180_json::serializer::{JsonSerializer, JsonSerializerOptions};
use crate::stdlib::srfi180_json::value_conversion::{json_to_scheme, scheme_to_json, JsonValue};

/// JSON Lines format processor
pub struct JsonLinesProcessor {
    options: JsonOptions,
    serializer_options: JsonSerializerOptions,
}

impl JsonLinesProcessor {
    /// Create a new JSON Lines processor
    pub fn new(options: JsonOptions, serializer_options: JsonSerializerOptions) -> Self {
        Self {
            options,
            serializer_options,
        }
    }
    
    /// Create with default options
    pub fn with_defaults() -> Self {
        Self {
            options: JsonOptions::default(),
            serializer_options: JsonSerializerOptions::default(),
        }
    }
    
    /// Parse JSON Lines format input into a vector of Scheme values
    pub fn parse_json_lines(&self, input: &str) -> JsonResult<Vec<Value>> {
        let mut results = Vec::new();
        
        for (line_number, line) in input.lines().enumerate() {
            let trimmed = line.trim();
            
            // Skip empty lines and comments (non-standard but useful)
            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }
            
            match parse_json_with_options(trimmed, self.options.clone()) {
                Ok(json_value) => {
                    let scheme_value = json_to_scheme(&json_value)?;
                    results.push(scheme_value);
                }
                Err(error) => {
                    // Add line number context to error
                    let contextual_error = JsonError::SyntaxError {
                        message: format!("JSON Lines error at line {}: {}", line_number + 1, error),
                        line: line_number + 1,
                        column: 1,
                        position: 0,
                    };
                    return Err(contextual_error);
                }
            }
        }
        
        Ok(results)
    }
    
    /// Serialize a vector of Scheme values to JSON Lines format
    pub fn serialize_json_lines(&self, values: &[Value]) -> JsonResult<String> {
        let mut output = String::new();
        let mut serializer = JsonSerializer::new(self.serializer_options.clone());
        
        for value in values {
            let json_string = serializer.serialize_scheme_value(value)?;
            output.push_str(&json_string);
            output.push('\n');
        }
        
        Ok(output)
    }
    
    /// Create a streaming parser for JSON Lines
    pub fn create_streaming_parser<'a>(&self, input: &'a str) -> JsonLinesStreamingParser<'a> {
        JsonLinesStreamingParser::new(input, self.options.clone())
    }
}

/// Streaming parser for JSON Lines format
pub struct JsonLinesStreamingParser<'a> {
    lines: std::str::Lines<'a>,
    current_line_number: usize,
    options: JsonOptions,
}

impl<'a> JsonLinesStreamingParser<'a> {
    /// Create a new streaming parser
    pub fn new(input: &'a str, options: JsonOptions) -> Self {
        Self {
            lines: input.lines(),
            current_line_number: 0,
            options,
        }
    }
    
    /// Parse the next JSON object from the stream
    pub fn next_value(&mut self) -> JsonResult<Option<Value>> {
        loop {
            match self.lines.next() {
                Some(line) => {
                    self.current_line_number += 1;
                    let trimmed = line.trim();
                    
                    // Skip empty lines and comments
                    if trimmed.is_empty() || trimmed.starts_with('#') {
                        continue;
                    }
                    
                    match parse_json_with_options(trimmed, self.options.clone()) {
                        Ok(json_value) => {
                            let scheme_value = json_to_scheme(&json_value)?;
                            return Ok(Some(scheme_value));
                        }
                        Err(error) => {
                            let contextual_error = JsonError::SyntaxError {
                                message: format!("JSON Lines error at line {}: {}", self.current_line_number, error),
                                line: self.current_line_number,
                                column: 1,
                                position: 0,
                            };
                            return Err(contextual_error);
                        }
                    }
                }
                None => return Ok(None), // End of input
            }
        }
    }
    
    /// Check if more values are available
    pub fn has_more(&self) -> bool {
        // This is an approximation - we'd need to peek at the iterator
        true // Conservative assumption
    }
}

/// JSON Text Sequences (RFC 7464) processor
pub struct JsonTextSequencesProcessor {
    options: JsonOptions,
    serializer_options: JsonSerializerOptions,
}

impl JsonTextSequencesProcessor {
    /// Record Separator character (ASCII 30, 0x1E)
    const RECORD_SEPARATOR: char = '\u{001E}';
    
    /// Create a new JSON Text Sequences processor
    pub fn new(options: JsonOptions, serializer_options: JsonSerializerOptions) -> Self {
        Self {
            options,
            serializer_options,
        }
    }
    
    /// Create with default options
    pub fn with_defaults() -> Self {
        Self {
            options: JsonOptions::default(),
            serializer_options: JsonSerializerOptions::default(),
        }
    }
    
    /// Parse JSON Text Sequences input into a vector of Scheme values
    pub fn parse_json_sequences(&self, input: &str) -> JsonResult<Vec<Value>> {
        let mut results = Vec::new();
        
        // Split on Record Separator (RS) character
        let records = input.split(Self::RECORD_SEPARATOR);
        
        for (record_number, record) in records.enumerate() {
            let trimmed = record.trim();
            
            // Skip empty records
            if trimmed.is_empty() {
                continue;
            }
            
            match parse_json_with_options(trimmed, self.options.clone()) {
                Ok(json_value) => {
                    let scheme_value = json_to_scheme(&json_value)?;
                    results.push(scheme_value);
                }
                Err(error) => {
                    let contextual_error = JsonError::SyntaxError {
                        message: format!("JSON Text Sequence error at record {}: {}", record_number + 1, error),
                        line: 1,
                        column: 1,
                        position: 0,
                    };
                    return Err(contextual_error);
                }
            }
        }
        
        Ok(results)
    }
    
    /// Serialize a vector of Scheme values to JSON Text Sequences format
    pub fn serialize_json_sequences(&self, values: &[Value]) -> JsonResult<String> {
        let mut output = String::new();
        let mut serializer = JsonSerializer::new(self.serializer_options.clone());
        
        for value in values {
            output.push(Self::RECORD_SEPARATOR);
            let json_string = serializer.serialize_scheme_value(value)?;
            output.push_str(&json_string);
        }
        
        Ok(output)
    }
    
    /// Create a streaming parser for JSON Text Sequences
    pub fn create_streaming_parser<'a>(&self, input: &'a str) -> JsonTextSequencesStreamingParser<'a> {
        JsonTextSequencesStreamingParser::new(input, self.options.clone())
    }
}

/// Streaming parser for JSON Text Sequences format
pub struct JsonTextSequencesStreamingParser<'a> {
    records: std::str::Split<'a, char>,
    current_record_number: usize,
    options: JsonOptions,
}

impl<'a> JsonTextSequencesStreamingParser<'a> {
    /// Create a new streaming parser
    pub fn new(input: &'a str, options: JsonOptions) -> Self {
        Self {
            records: input.split(JsonTextSequencesProcessor::RECORD_SEPARATOR),
            current_record_number: 0,
            options,
        }
    }
    
    /// Parse the next JSON value from the stream
    pub fn next_value(&mut self) -> JsonResult<Option<Value>> {
        loop {
            match self.records.next() {
                Some(record) => {
                    self.current_record_number += 1;
                    let trimmed = record.trim();
                    
                    // Skip empty records
                    if trimmed.is_empty() {
                        continue;
                    }
                    
                    match parse_json_with_options(trimmed, self.options.clone()) {
                        Ok(json_value) => {
                            let scheme_value = json_to_scheme(&json_value)?;
                            return Ok(Some(scheme_value));
                        }
                        Err(error) => {
                            let contextual_error = JsonError::SyntaxError {
                                message: format!("JSON Text Sequence error at record {}: {}", 
                                    self.current_record_number, error),
                                line: 1,
                                column: 1,
                                position: 0,
                            };
                            return Err(contextual_error);
                        }
                    }
                }
                None => return Ok(None), // End of input
            }
        }
    }
    
    /// Check if more values are available
    pub fn has_more(&self) -> bool {
        // Conservative assumption
        true
    }
}

/// Convenience functions for common use cases

/// Parse JSON Lines format with default options
pub fn parse_json_lines(input: &str) -> JsonResult<Vec<Value>> {
    let processor = JsonLinesProcessor::with_defaults();
    processor.parse_json_lines(input)
}

/// Serialize to JSON Lines format with default options
pub fn serialize_json_lines(values: &[Value]) -> JsonResult<String> {
    let processor = JsonLinesProcessor::with_defaults();
    processor.serialize_json_lines(values)
}

/// Parse JSON Text Sequences with default options
pub fn parse_json_sequences(input: &str) -> JsonResult<Vec<Value>> {
    let processor = JsonTextSequencesProcessor::with_defaults();
    processor.parse_json_sequences(input)
}

/// Serialize to JSON Text Sequences format with default options
pub fn serialize_json_sequences(values: &[Value]) -> JsonResult<String> {
    let processor = JsonTextSequencesProcessor::with_defaults();
    processor.serialize_json_sequences(values)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::eval::value::Value;
    
    #[test]
    fn test_json_lines_parsing() {
        let input = r#"{"name": "Alice", "age": 30}
{"name": "Bob", "age": 25}
{"name": "Charlie", "age": 35}"#;
        
        let values = parse_json_lines(input).unwrap();
        assert_eq!(values.len(), 3);
    }
    
    #[test]
    fn test_json_lines_with_empty_lines() {
        let input = r#"{"name": "Alice"}

{"name": "Bob"}
        
{"name": "Charlie"}"#;
        
        let values = parse_json_lines(input).unwrap();
        assert_eq!(values.len(), 3);
    }
    
    #[test]
    fn test_json_lines_serialization() {
        let values = vec![
            Value::string("hello"),
            Value::integer(42),
            Value::boolean(true),
        ];
        
        let json_lines = serialize_json_lines(&values).unwrap();
        let expected = "\"hello\"\n42\ntrue\n";
        assert_eq!(json_lines, expected);
    }
    
    #[test]
    fn test_json_text_sequences_parsing() {
        let input = format!(
            "{}{{\"name\": \"Alice\"}}{}{{\"name\": \"Bob\"}}",
            JsonTextSequencesProcessor::RECORD_SEPARATOR,
            JsonTextSequencesProcessor::RECORD_SEPARATOR
        );
        
        let values = parse_json_sequences(&input).unwrap();
        assert_eq!(values.len(), 2);
    }
    
    #[test]
    fn test_json_text_sequences_serialization() {
        let values = vec![
            Value::string("hello"),
            Value::integer(42),
        ];
        
        let json_sequences = serialize_json_sequences(&values).unwrap();
        assert!(json_sequences.starts_with('\u{001E}'));
        assert!(json_sequences.contains("\"hello\""));
        assert!(json_sequences.contains("42"));
    }
    
    #[test]
    fn test_json_lines_streaming() {
        let input = r#"{"a": 1}
{"b": 2}
{"c": 3}"#;
        
        let processor = JsonLinesProcessor::with_defaults();
        let mut parser = processor.create_streaming_parser(input);
        
        let mut count = 0;
        while let Some(_value) = parser.next_value().unwrap() {
            count += 1;
        }
        
        assert_eq!(count, 3);
    }
}