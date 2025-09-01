//! JSON parser for SRFI-180
//!
//! This module provides a recursive descent parser for JSON with streaming
//! support and security features (depth/size limits).

use crate::stdlib::srfi180_json::error::{JsonError, JsonOptions, JsonResult};
use crate::stdlib::srfi180_json::lexer::{JsonLexer, JsonToken};
use crate::stdlib::srfi180_json::value_conversion::{JsonNumber, JsonValue, parse_json_number};
use std::collections::HashMap;

/// JSON parser with streaming support and security limits
pub struct JsonParser<'a> {
    lexer: JsonLexer<'a>,
    current_token: JsonToken,
    current_depth: usize,
    options: JsonOptions,
}

impl<'a> JsonParser<'a> {
    /// Create a new JSON parser with input string and options
    pub fn new(input: &'a str, options: JsonOptions) -> JsonResult<Self> {
        let mut lexer = JsonLexer::new(input);
        let current_token = lexer.next_token()?;
        
        Ok(Self {
            lexer,
            current_token,
            current_depth: 0,
            options,
        })
    }
    
    /// Create a new JSON parser with default options
    pub fn with_defaults(input: &'a str) -> JsonResult<Self> {
        Self::new(input, JsonOptions::default())
    }
    
    /// Advance to the next token
    fn next_token(&mut self) -> JsonResult<()> {
        self.current_token = self.lexer.next_token()?;
        Ok(())
    }
    
    /// Check for depth limit
    fn check_depth_limit(&self) -> JsonResult<()> {
        if self.current_depth >= self.options.max_depth {
            return Err(JsonError::depth_limit_exceeded(self.options.max_depth, self.current_depth));
        }
        Ok(())
    }
    
    /// Parse a complete JSON value
    pub fn parse(&mut self) -> JsonResult<JsonValue> {
        let value = self.parse_value()?;
        
        // Ensure we've consumed all input
        match self.current_token {
            JsonToken::Eof => Ok(value),
            _ => Err(JsonError::syntax_error(
                format!("unexpected token after JSON value: {}", self.current_token),
                self.lexer.position().line,
                self.lexer.position().column,
            )),
        }
    }
    
    /// Parse any JSON value
    fn parse_value(&mut self) -> JsonResult<JsonValue> {
        match &self.current_token {
            JsonToken::LeftBrace => self.parse_object(),
            JsonToken::LeftBracket => self.parse_array(),
            JsonToken::String(s) => {
                let value = JsonValue::String(s.clone());
                self.next_token()?;
                Ok(value)
            }
            JsonToken::Number(n) => {
                let json_number = parse_json_number(&n.to_string())?;
                let value = JsonValue::number(json_number);
                self.next_token()?;
                Ok(value)
            }
            JsonToken::True => {
                let value = JsonValue::boolean(true);
                self.next_token()?;
                Ok(value)
            }
            JsonToken::False => {
                let value = JsonValue::boolean(false);
                self.next_token()?;
                Ok(value)
            }
            JsonToken::Null => {
                let value = JsonValue::Null;
                self.next_token()?;
                Ok(value)
            }
            token => Err(JsonError::syntax_error(
                format!("unexpected token: {}", token),
                self.lexer.position().line,
                self.lexer.position().column,
            )),
        }
    }
    
    /// Parse a JSON object
    fn parse_object(&mut self) -> JsonResult<JsonValue> {
        self.check_depth_limit()?;
        self.current_depth += 1;
        
        let mut object = HashMap::new();
        
        // Consume opening brace
        self.next_token()?;
        
        // Handle empty object
        if matches!(self.current_token, JsonToken::RightBrace) {
            self.next_token()?;
            self.current_depth -= 1;
            return Ok(JsonValue::Object(object));
        }
        
        loop {
            // Parse key (must be a string)
            let key = match &self.current_token {
                JsonToken::String(s) => s.clone(),
                token => {
                    return Err(JsonError::syntax_error(
                        format!("expected string key, found: {}", token),
                        self.lexer.position().line,
                        self.lexer.position().column,
                    ));
                }
            };
            
            self.next_token()?;
            
            // Expect colon
            match &self.current_token {
                JsonToken::Colon => {
                    self.next_token()?;
                }
                token => {
                    return Err(JsonError::syntax_error(
                        format!("expected ':', found: {}", token),
                        self.lexer.position().line,
                        self.lexer.position().column,
                    ));
                }
            }
            
            // Parse value
            let value = self.parse_value()?;
            object.insert(key, value);
            
            // Check for comma or end
            match &self.current_token {
                JsonToken::Comma => {
                    self.next_token()?;
                    // Disallow trailing commas in strict mode
                    if self.options.strict_mode && matches!(self.current_token, JsonToken::RightBrace) {
                        return Err(JsonError::syntax_error(
                            "trailing comma in object".to_string(),
                            self.lexer.position().line,
                            self.lexer.position().column,
                        ));
                    }
                }
                JsonToken::RightBrace => {
                    self.next_token()?;
                    break;
                }
                token => {
                    return Err(JsonError::syntax_error(
                        format!("expected ',' or '}}', found: {}", token),
                        self.lexer.position().line,
                        self.lexer.position().column,
                    ));
                }
            }
        }
        
        self.current_depth -= 1;
        Ok(JsonValue::Object(object))
    }
    
    /// Parse a JSON array
    fn parse_array(&mut self) -> JsonResult<JsonValue> {
        self.check_depth_limit()?;
        self.current_depth += 1;
        
        let mut array = Vec::new();
        
        // Consume opening bracket
        self.next_token()?;
        
        // Handle empty array
        if matches!(self.current_token, JsonToken::RightBracket) {
            self.next_token()?;
            self.current_depth -= 1;
            return Ok(JsonValue::Array(array));
        }
        
        loop {
            // Parse value
            let value = self.parse_value()?;
            array.push(value);
            
            // Check for comma or end
            match &self.current_token {
                JsonToken::Comma => {
                    self.next_token()?;
                    // Disallow trailing commas in strict mode
                    if self.options.strict_mode && matches!(self.current_token, JsonToken::RightBracket) {
                        return Err(JsonError::syntax_error(
                            "trailing comma in array".to_string(),
                            self.lexer.position().line,
                            self.lexer.position().column,
                        ));
                    }
                }
                JsonToken::RightBracket => {
                    self.next_token()?;
                    break;
                }
                token => {
                    return Err(JsonError::syntax_error(
                        format!("expected ',' or ']', found: {}", token),
                        self.lexer.position().line,
                        self.lexer.position().column,
                    ));
                }
            }
        }
        
        self.current_depth -= 1;
        Ok(JsonValue::Array(array))
    }
    
    /// Get current position for error reporting
    pub fn position(&self) -> &crate::stdlib::srfi180_json::error::Position {
        self.lexer.position()
    }
}

/// Parse JSON string with default options
pub fn parse_json(input: &str) -> JsonResult<JsonValue> {
    let mut parser = JsonParser::with_defaults(input)?;
    parser.parse()
}

/// Parse JSON string with custom options
pub fn parse_json_with_options(input: &str, options: JsonOptions) -> JsonResult<JsonValue> {
    let mut parser = JsonParser::new(input, options)?;
    parser.parse()
}

/// Streaming JSON parser for large documents
pub struct StreamingJsonParser<'a> {
    parser: JsonParser<'a>,
}

impl<'a> StreamingJsonParser<'a> {
    /// Create a new streaming parser
    pub fn new(input: &'a str, options: JsonOptions) -> JsonResult<Self> {
        let parser = JsonParser::new(input, options)?;
        Ok(Self { parser })
    }
    
    /// Parse the next top-level JSON value
    pub fn next_value(&mut self) -> JsonResult<Option<JsonValue>> {
        match self.parser.current_token {
            JsonToken::Eof => Ok(None),
            _ => {
                let value = self.parser.parse_value()?;
                Ok(Some(value))
            }
        }
    }
    
    /// Check if more values are available
    pub fn has_more(&self) -> bool {
        !matches!(self.parser.current_token, JsonToken::Eof)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_parse_null() {
        let result = parse_json("null").unwrap();
        assert_eq!(result, JsonValue::Null);
    }
    
    #[test]
    fn test_parse_boolean() {
        let result = parse_json("true").unwrap();
        assert_eq!(result, JsonValue::boolean(true));
        
        let result = parse_json("false").unwrap();
        assert_eq!(result, JsonValue::boolean(false));
    }
    
    #[test]
    fn test_parse_number() {
        let result = parse_json("42").unwrap();
        assert_eq!(result, JsonValue::number(JsonNumber::Integer(42)));
        
        let result = parse_json("3.14159").unwrap();
        assert_eq!(result, JsonValue::number(JsonNumber::Float(3.14159)));
    }
    
    #[test]
    fn test_parse_string() {
        let result = parse_json(r#""hello world""#).unwrap();
        assert_eq!(result, JsonValue::String("hello world".to_string()));
    }
    
    #[test]
    fn test_parse_empty_array() {
        let result = parse_json("[]").unwrap();
        assert_eq!(result, JsonValue::Array(vec![]));
    }
    
    #[test]
    fn test_parse_simple_array() {
        let result = parse_json("[1, 2, 3]").unwrap();
        assert_eq!(result, JsonValue::Array(vec![
            JsonValue::number(JsonNumber::Integer(1)),
            JsonValue::number(JsonNumber::Integer(2)),
            JsonValue::number(JsonNumber::Integer(3)),
        ]));
    }
    
    #[test]
    fn test_parse_empty_object() {
        let result = parse_json("{}").unwrap();
        assert_eq!(result, JsonValue::Object(HashMap::new()));
    }
    
    #[test]
    fn test_parse_simple_object() {
        let result = parse_json(r#"{"name": "Alice", "age": 30}"#).unwrap();
        
        let mut expected = HashMap::new();
        expected.insert("name".to_string(), JsonValue::String("Alice".to_string()));
        expected.insert("age".to_string(), JsonValue::number(JsonNumber::Integer(30)));
        
        assert_eq!(result, JsonValue::Object(expected));
    }
    
    #[test]
    fn test_parse_nested_structure() {
        let result = parse_json(r#"{
            "users": [
                {"name": "Alice", "active": true},
                {"name": "Bob", "active": false}
            ],
            "count": 2
        }"#).unwrap();
        
        match result {
            JsonValue::Object(obj) => {
                assert!(obj.contains_key("users"));
                assert!(obj.contains_key("count"));
            }
            _ => panic!("Expected object"),
        }
    }
    
    #[test]
    fn test_depth_limit() {
        let options = JsonOptions::default().with_max_depth(2);
        let deep_json = "[[[1]]]"; // Depth 3
        
        let result = parse_json_with_options(deep_json, options);
        assert!(result.is_err());
        
        match result.unwrap_err().kind() {
            crate::stdlib::srfi180_json::error::JsonErrorKind::DepthLimitExceeded => {
                // Just verify we got the right error type
            }
            _ => panic!("Expected depth limit error"),
        }
    }
    
    // TODO: Add strict mode functionality with trailing comma detection
    // #[test]
    // fn test_trailing_comma_strict() {
    //     let options = JsonOptions::default().with_strict_mode(true);
    //     // ... test implementation
    // }
}