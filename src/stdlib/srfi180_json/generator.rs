//! JSON generator for SRFI-180 streaming support
//!
//! This module provides generator-based JSON parsing for handling large
//! JSON documents efficiently with constant memory usage.

use crate::containers::generator::{Generator, GeneratorState, EvaluatorRef};
use crate::eval::value::{ThreadSafeEnvironment, Value};
use crate::stdlib::srfi180_json::error::{JsonError, JsonOptions, JsonResult};
use crate::stdlib::srfi180_json::lexer::{JsonLexer, JsonToken};
use crate::stdlib::srfi180_json::value_conversion::{JsonValue, json_to_scheme, parse_json_number};
use std::collections::HashMap;
use std::sync::Arc;

/// JSON generator state for streaming parsing
#[derive(Debug, Clone)]
pub enum JsonGeneratorState {
    /// Parsing array elements
    ArrayElement {
        lexer_state: String, // Serialized lexer state
        current_token: JsonToken,
        depth: usize,
    },
    
    /// Parsing object key-value pairs  
    ObjectPair {
        lexer_state: String,
        current_token: JsonToken,
        depth: usize,
    },
    
    /// Single value (complete)
    SingleValue {
        value: JsonValue,
        consumed: bool,
    },
    
    /// End of stream
    Exhausted,
}

/// Generator for streaming JSON parsing
pub struct JsonGenerator {
    state: JsonGeneratorState,
    options: JsonOptions,
    environment: Arc<ThreadSafeEnvironment>,
    evaluator: EvaluatorRef,
}

impl JsonGenerator {
    /// Create a new JSON generator from input string
    pub fn new(
        input: &str,
        options: JsonOptions,
        environment: Arc<ThreadSafeEnvironment>,
        evaluator: EvaluatorRef,
    ) -> JsonResult<Self> {
        let mut lexer = JsonLexer::new(input);
        let first_token = lexer.next_token()?;
        
        let state = match first_token {
            JsonToken::LeftBracket => {
                // Array stream
                JsonGeneratorState::ArrayElement {
                    lexer_state: Self::serialize_lexer_state(&lexer),
                    current_token: first_token,
                    depth: 0,
                }
            }
            JsonToken::LeftBrace => {
                // Object stream  
                JsonGeneratorState::ObjectPair {
                    lexer_state: Self::serialize_lexer_state(&lexer),
                    current_token: first_token,
                    depth: 0,
                }
            }
            JsonToken::Eof => {
                JsonGeneratorState::Exhausted
            }
            _ => {
                // Single value
                let value = Self::parse_single_value(first_token, &mut lexer)?;
                JsonGeneratorState::SingleValue {
                    value,
                    consumed: false,
                }
            }
        };
        
        Ok(Self {
            state,
            options,
            environment,
            evaluator,
        })
    }
    
    /// Parse a single JSON token into a value
    fn parse_single_value(token: JsonToken, lexer: &mut JsonLexer<'_>) -> JsonResult<JsonValue> {
        match token {
            JsonToken::String(s) => Ok(JsonValue::String(s)),
            JsonToken::Number(n) => {
                let json_number = parse_json_number(&n.to_string())?;
                Ok(JsonValue::number(json_number))
            }
            JsonToken::True => Ok(JsonValue::boolean(true)),
            JsonToken::False => Ok(JsonValue::boolean(false)),
            JsonToken::Null => Ok(JsonValue::Null),
            _ => Err(JsonError::SyntaxError {
                message: format!("unexpected token: {}", token),
                line: lexer.position().line,
                column: lexer.position().column,
                position: lexer.position().offset,
            }),
        }
    }
    
    /// Serialize lexer state (simplified - in real implementation would need proper serialization)
    fn serialize_lexer_state(lexer: &JsonLexer<'_>) -> String {
        format!("{}:{}:{}", lexer.position().line, lexer.position().column, lexer.position().offset)
    }
    
    /// Generate the next JSON value from the stream
    pub fn next_value(&mut self) -> JsonResult<Option<Value>> {
        match &mut self.state {
            JsonGeneratorState::SingleValue { value, consumed } => {
                if *consumed {
                    Ok(None)
                } else {
                    *consumed = true;
                    let scheme_value = json_to_scheme(value)?;
                    Ok(Some(scheme_value))
                }
            }
            JsonGeneratorState::ArrayElement { .. } => {
                // TODO: Implement array element streaming
                // This would involve maintaining lexer state and parsing one element at a time
                Err(JsonError::conversion_error(
                    "streaming array",
                    "value",
                    "array streaming not yet implemented",
                ))
            }
            JsonGeneratorState::ObjectPair { .. } => {
                // TODO: Implement object pair streaming  
                // This would involve maintaining lexer state and parsing one key-value pair at a time
                Err(JsonError::conversion_error(
                    "streaming object",
                    "value", 
                    "object streaming not yet implemented",
                ))
            }
            JsonGeneratorState::Exhausted => Ok(None),
        }
    }
    
    /// Check if the generator has more values
    pub fn has_more(&self) -> bool {
        match &self.state {
            JsonGeneratorState::Exhausted => false,
            JsonGeneratorState::SingleValue { consumed, .. } => !consumed,
            _ => true, // Array and object streams may have more
        }
    }
    
    /// Convert to Lambdust Generator compatible with SRFI-121
    pub fn to_lambdust_generator(self) -> Value {
        let generator_state = GeneratorState::Procedure {
            thunk: Value::Primitive(Arc::new(crate::eval::value::PrimitiveProcedure {
                name: "json-generator-thunk".to_string(),
                arity_min: 0,
                arity_max: Some(0),
                implementation: crate::eval::value::PrimitiveImpl::RustFn(|_args| {
                    // This is a simplified version - real implementation would need
                    // proper closure capture and state management
                    Ok(Value::symbol_from_str("*eof-object*"))
                }),
                effects: vec![crate::effects::Effect::Pure],
            })),
            environment: self.environment.clone(),
            evaluator: self.evaluator,
        };
        
        
        let thunk = Value::Primitive(Arc::new(crate::eval::value::PrimitiveProcedure {
            name: "json-generator-thunk".to_string(),
            arity_min: 0,
            arity_max: Some(0),
            implementation: crate::eval::value::PrimitiveImpl::RustFn(|_args| {
                // This is a simplified version - real implementation would need
                // proper closure capture and state management
                Ok(Value::symbol_from_str("*eof-object*"))
            }),
            effects: vec![crate::effects::Effect::Pure],
        }));
        
        let mut generator = Generator::from_procedure(thunk, self.environment.clone());
        generator.set_name("json-generator".to_string());
        Value::Generator(Arc::new(generator))
    }
}

/// JSON fold operation for streaming processing
pub struct JsonFolder<F> {
    generator: JsonGenerator,
    folder_fn: F,
    accumulator: Value,
}

impl<F> JsonFolder<F>
where
    F: Fn(&Value, &Value) -> JsonResult<Value>,
{
    /// Create a new JSON folder
    pub fn new(generator: JsonGenerator, folder_fn: F, initial_value: Value) -> Self {
        Self {
            generator,
            folder_fn,
            accumulator: initial_value,
        }
    }
    
    /// Fold over all values in the JSON stream
    pub fn fold_all(mut self) -> JsonResult<Value> {
        while let Some(value) = self.generator.next_value()? {
            self.accumulator = (self.folder_fn)(&self.accumulator, &value)?;
        }
        Ok(self.accumulator)
    }
    
    /// Fold over the next N values
    pub fn fold_n(&mut self, n: usize) -> JsonResult<Value> {
        for _ in 0..n {
            if let Some(value) = self.generator.next_value()? {
                self.accumulator = (self.folder_fn)(&self.accumulator, &value)?;
            } else {
                break;
            }
        }
        Ok(self.accumulator.clone())
    }
}

/// Create a JSON generator from input with default options
pub fn json_generator_from_string(
    input: &str,
    environment: Arc<ThreadSafeEnvironment>,
) -> JsonResult<JsonGenerator> {
    let options = JsonOptions::default();
    let evaluator = EvaluatorRef::new();
    JsonGenerator::new(input, options, environment, evaluator)
}

/// Create a JSON generator with custom options
pub fn json_generator_with_options(
    input: &str,
    options: JsonOptions,
    environment: Arc<ThreadSafeEnvironment>,
    evaluator: EvaluatorRef,
) -> JsonResult<JsonGenerator> {
    JsonGenerator::new(input, options, environment, evaluator)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::eval::value::ThreadSafeEnvironment;
    
    #[test]
    fn test_single_value_generator() {
        let env = Arc::new(ThreadSafeEnvironment::new(None, 0));
        let mut generator = json_generator_from_string("42", env).unwrap();
        
        // Should get one value
        let value = generator.next_value().unwrap();
        assert!(value.is_some());
        
        // Should be exhausted
        let value = generator.next_value().unwrap();
        assert!(value.is_none());
    }
    
    #[test]
    fn test_generator_has_more() {
        let env = Arc::new(ThreadSafeEnvironment::new(None, 0));
        let mut generator = json_generator_from_string("true", env).unwrap();
        
        assert!(generator.has_more());
        
        let _value = generator.next_value().unwrap();
        assert!(!generator.has_more());
    }
    
    #[test]
    fn test_empty_generator() {
        let env = Arc::new(ThreadSafeEnvironment::new(None, 0));
        let mut generator = json_generator_from_string("", env).unwrap();
        
        assert!(!generator.has_more());
        
        let value = generator.next_value().unwrap();
        assert!(value.is_none());
    }
}