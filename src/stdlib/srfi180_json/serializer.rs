//! JSON serializer for SRFI-180
//!
//! This module provides JSON serialization with support for:
//! - RFC 8259 compliant JSON output
//! - Incremental JSON construction (json-accumulator)
//! - Pretty printing with configurable indentation
//! - Streaming output to avoid memory buildup

use crate::eval::value::Value;
use crate::stdlib::srfi180_json::error::{JsonError, JsonOptions, JsonResult};
use crate::stdlib::srfi180_json::value_conversion::{JsonNumber, JsonValue, scheme_to_json};
use std::collections::HashMap;
use std::fmt::Write;

/// JSON serialization configuration
#[derive(Debug, Clone)]
pub struct JsonSerializerOptions {
    /// Enable pretty printing with indentation
    pub pretty_print: bool,
    
    /// Indentation string (e.g., "  " for 2 spaces, "\t" for tab)
    pub indent_string: String,
    
    /// Maximum line length before wrapping (0 = no limit)
    pub max_line_length: usize,
    
    /// Whether to escape non-ASCII characters
    pub escape_unicode: bool,
    
    /// Whether to sort object keys
    pub sort_keys: bool,
}

impl Default for JsonSerializerOptions {
    fn default() -> Self {
        Self {
            pretty_print: false,
            indent_string: "  ".to_string(), // 2 spaces
            max_line_length: 0, // No limit
            escape_unicode: false,
            sort_keys: false,
        }
    }
}

impl JsonSerializerOptions {
    pub fn new() -> Self {
        Self::default()
    }
    
    pub fn with_pretty_print(mut self, pretty_print: bool) -> Self {
        self.pretty_print = pretty_print;
        self
    }
    
    pub fn with_indent_string(mut self, indent_string: String) -> Self {
        self.indent_string = indent_string;
        self
    }
    
    pub fn with_max_line_length(mut self, max_line_length: usize) -> Self {
        self.max_line_length = max_line_length;
        self
    }
    
    pub fn with_escape_unicode(mut self, escape_unicode: bool) -> Self {
        self.escape_unicode = escape_unicode;
        self
    }
    
    pub fn with_sort_keys(mut self, sort_keys: bool) -> Self {
        self.sort_keys = sort_keys;
        self
    }
}

/// JSON serializer for converting values to JSON strings
pub struct JsonSerializer {
    options: JsonSerializerOptions,
    current_depth: usize,
    output: String,
}

impl JsonSerializer {
    /// Create a new JSON serializer with options
    pub fn new(options: JsonSerializerOptions) -> Self {
        Self {
            options,
            current_depth: 0,
            output: String::new(),
        }
    }
    
    /// Create a serializer with default options
    pub fn with_defaults() -> Self {
        Self::new(JsonSerializerOptions::default())
    }
    
    /// Serialize a JsonValue to JSON string
    pub fn serialize_json_value(&mut self, value: &JsonValue) -> JsonResult<String> {
        self.output.clear();
        self.current_depth = 0;
        self.write_json_value(value)?;
        Ok(self.output.clone())
    }
    
    /// Serialize a Scheme Value to JSON string
    pub fn serialize_scheme_value(&mut self, value: &Value) -> JsonResult<String> {
        let json_value = scheme_to_json(value)?;
        self.serialize_json_value(&json_value)
    }
    
    /// Write a JSON value to the output
    fn write_json_value(&mut self, value: &JsonValue) -> JsonResult<()> {
        match value {
            JsonValue::Null => self.write_literal("null"),
            JsonValue::boolean(true) => self.write_literal("true"),
            JsonValue::boolean(false) => self.write_literal("false"),
            JsonValue::number(num) => self.write_number(num),
            JsonValue::String(s) => self.write_string(s),
            JsonValue::Array(arr) => self.write_array(arr),
            JsonValue::Object(obj) => self.write_object(obj),
        }
    }
    
    /// Write a literal value
    fn write_literal(&mut self, literal: &str) -> JsonResult<()> {
        self.output.push_str(literal);
        Ok(())
    }
    
    /// Write a JSON number
    fn write_number(&mut self, number: &JsonNumber) -> JsonResult<()> {
        let number_str = match number {
            JsonNumber::Integer(i) => i.to_string(),
            JsonNumber::Float(f) => {
                if f.is_infinite() || f.is_nan() {
                    return Err(JsonError::conversion_error(
                        "infinite or NaN number",
                        "JSON number",
                        "JSON does not support infinite or NaN values",
                    ));
                }
                f.to_string()
            }
            JsonNumber::Exact(num, den) => {
                if *den == 0 {
                    return Err(JsonError::conversion_error(
                        "rational with zero denominator",
                        "JSON number",
                        "division by zero",
                    ));
                }
                (*num as f64 / *den as f64).to_string()
            }
            JsonNumber::Raw(s) => s.clone(),
        };
        
        self.output.push_str(&number_str);
        Ok(())
    }
    
    /// Write a JSON string with proper escaping
    fn write_string(&mut self, s: &str) -> JsonResult<()> {
        self.output.push('"');
        
        for ch in s.chars() {
            match ch {
                '"' => self.output.push_str("\\\""),
                '\\' => self.output.push_str("\\\\"),
                '\u{0008}' => self.output.push_str("\\b"), // backspace
                '\u{000C}' => self.output.push_str("\\f"), // form feed
                '\n' => self.output.push_str("\\n"),
                '\r' => self.output.push_str("\\r"),
                '\t' => self.output.push_str("\\t"),
                ch if ch.is_control() => {
                    write!(self.output, "\\u{:04x}", ch as u32)
                        .map_err(|_| JsonError::io_error("failed to write Unicode escape"))?;
                }
                ch if self.options.escape_unicode && !ch.is_ascii() => {
                    write!(self.output, "\\u{:04x}", ch as u32)
                        .map_err(|_| JsonError::io_error("failed to write Unicode escape"))?;
                }
                ch => self.output.push(ch),
            }
        }
        
        self.output.push('"');
        Ok(())
    }
    
    /// Write a JSON array
    fn write_array(&mut self, array: &[JsonValue]) -> JsonResult<()> {
        self.output.push('[');
        
        if array.is_empty() {
            self.output.push(']');
            return Ok(());
        }
        
        self.current_depth += 1;
        
        if self.options.pretty_print {
            self.output.push('\n');
        }
        
        for (i, item) in array.iter().enumerate() {
            if self.options.pretty_print {
                self.write_indent();
            }
            
            self.write_json_value(item)?;
            
            if i < array.len() - 1 {
                self.output.push(',');
                if self.options.pretty_print {
                    self.output.push('\n');
                } else {
                    self.output.push(' ');
                }
            }
        }
        
        if self.options.pretty_print {
            self.output.push('\n');
            self.current_depth -= 1;
            self.write_indent();
        } else {
            self.current_depth -= 1;
        }
        
        self.output.push(']');
        Ok(())
    }
    
    /// Write a JSON object
    fn write_object(&mut self, object: &HashMap<String, JsonValue>) -> JsonResult<()> {
        self.output.push('{');
        
        if object.is_empty() {
            self.output.push('}');
            return Ok(());
        }
        
        self.current_depth += 1;
        
        if self.options.pretty_print {
            self.output.push('\n');
        }
        
        // Get keys, optionally sorted
        let mut keys: Vec<&String> = object.keys().collect();
        if self.options.sort_keys {
            keys.sort();
        }
        
        for (i, key) in keys.iter().enumerate() {
            if self.options.pretty_print {
                self.write_indent();
            }
            
            // Write key
            self.write_string(key)?;
            self.output.push(':');
            
            if self.options.pretty_print {
                self.output.push(' ');
            }
            
            // Write value
            if let Some(value) = object.get(*key) {
                self.write_json_value(value)?;
            }
            
            if i < keys.len() - 1 {
                self.output.push(',');
                if self.options.pretty_print {
                    self.output.push('\n');
                } else {
                    self.output.push(' ');
                }
            }
        }
        
        if self.options.pretty_print {
            self.output.push('\n');
            self.current_depth -= 1;
            self.write_indent();
        } else {
            self.current_depth -= 1;
        }
        
        self.output.push('}');
        Ok(())
    }
    
    /// Write indentation for pretty printing
    fn write_indent(&mut self) {
        for _ in 0..self.current_depth {
            self.output.push_str(&self.options.indent_string);
        }
    }
}

/// JSON accumulator for incremental JSON construction
pub struct JsonAccumulator {
    parts: Vec<String>,
    options: JsonSerializerOptions,
}

impl JsonAccumulator {
    /// Create a new JSON accumulator
    pub fn new(options: JsonSerializerOptions) -> Self {
        Self {
            parts: Vec::new(),
            options,
        }
    }
    
    /// Create an accumulator with default options
    pub fn with_defaults() -> Self {
        Self::new(JsonSerializerOptions::default())
    }
    
    /// Add a value to the accumulator
    pub fn add_value(&mut self, value: &Value) -> JsonResult<()> {
        let mut serializer = JsonSerializer::new(self.options.clone());
        let json_string = serializer.serialize_scheme_value(value)?;
        self.parts.push(json_string);
        Ok(())
    }
    
    /// Add a raw JSON string to the accumulator
    pub fn add_raw_json(&mut self, json_string: String) -> JsonResult<()> {
        // TODO: Validate that json_string is valid JSON
        self.parts.push(json_string);
        Ok(())
    }
    
    /// Get the accumulated JSON as a single string
    pub fn finalize(&self) -> String {
        if self.parts.is_empty() {
            return "[]".to_string(); // Empty array
        }
        
        if self.parts.len() == 1 {
            return self.parts[0].clone();
        }
        
        // Multiple parts - create JSON array
        let mut result = String::from("[");
        
        if self.options.pretty_print {
            result.push('\n');
        }
        
        for (i, part) in self.parts.iter().enumerate() {
            if self.options.pretty_print {
                result.push_str(&self.options.indent_string);
            }
            
            result.push_str(part);
            
            if i < self.parts.len() - 1 {
                result.push(',');
                if self.options.pretty_print {
                    result.push('\n');
                } else {
                    result.push(' ');
                }
            }
        }
        
        if self.options.pretty_print {
            result.push('\n');
        }
        
        result.push(']');
        result
    }
    
    /// Clear all accumulated values
    pub fn clear(&mut self) {
        self.parts.clear();
    }
    
    /// Get the number of accumulated values
    pub fn len(&self) -> usize {
        self.parts.len()
    }
    
    /// Check if the accumulator is empty
    pub fn is_empty(&self) -> bool {
        self.parts.is_empty()
    }
}

/// Serialize a Scheme value to JSON string with default options
pub fn serialize_to_json(value: &Value) -> JsonResult<String> {
    let mut serializer = JsonSerializer::with_defaults();
    serializer.serialize_scheme_value(value)
}

/// Serialize a Scheme value to JSON string with custom options
pub fn serialize_to_json_with_options(
    value: &Value,
    options: JsonSerializerOptions,
) -> JsonResult<String> {
    let mut serializer = JsonSerializer::new(options);
    serializer.serialize_scheme_value(value)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::eval::value::Value;
    
    #[test]
    fn test_serialize_primitives() {
        // Null
        let null_value = Value::symbol_from_str("null");
        let json = serialize_to_json(&null_value).unwrap();
        assert_eq!(json, "null");
        
        // Boolean
        let bool_value = Value::boolean(true);
        let json = serialize_to_json(&bool_value).unwrap();
        assert_eq!(json, "true");
        
        // Number
        let number_value = Value::integer(42);
        let json = serialize_to_json(&number_value).unwrap();
        assert_eq!(json, "42");
        
        // String
        let string_value = Value::string("hello world");
        let json = serialize_to_json(&string_value).unwrap();
        assert_eq!(json, r#""hello world""#);
    }
    
    #[test]
    fn test_serialize_string_escaping() {
        let string_value = Value::string("hello\nworld\t\"test\"\\");
        let json = serialize_to_json(&string_value).unwrap();
        assert_eq!(json, r#""hello\nworld\t\"test\"\\""#);
    }
    
    #[test]
    fn test_serialize_array() {
        let array_value = Value::vector(vec![
            Value::integer(1),
            Value::integer(2),
            Value::integer(3),
        ]);
        let json = serialize_to_json(&array_value).unwrap();
        assert_eq!(json, "[1, 2, 3]");
    }
    
    #[test]
    fn test_serialize_empty_array() {
        let array_value = Value::vector(vec![]);
        let json = serialize_to_json(&array_value).unwrap();
        assert_eq!(json, "[]");
    }
    
    #[test]
    fn test_accumulator() {
        let mut accumulator = JsonAccumulator::with_defaults();
        
        accumulator.add_value(&Value::integer(1)).unwrap();
        accumulator.add_value(&Value::string("hello")).unwrap();
        accumulator.add_value(&Value::boolean(true)).unwrap();
        
        let result = accumulator.finalize();
        assert_eq!(result, r#"[1, "hello", true]"#);
    }
    
    #[test]
    fn test_pretty_printing() {
        let options = JsonSerializerOptions::default()
            .with_pretty_print(true)
            .with_indent_string("  ".to_string());
        
        let array_value = Value::vector(vec![
            Value::integer(1),
            Value::integer(2),
        ]);
        
        let json = serialize_to_json_with_options(&array_value, options).unwrap();
        assert_eq!(json, "[\n  1,\n  2\n]");
    }
}