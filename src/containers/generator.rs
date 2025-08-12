//! SRFI-121: Generators - Core infrastructure
//!
//! This module provides the core generator implementation for SRFI-121.
//! Generators are stateful procedures that can be invoked repeatedly to
//! produce a sequence of values. They support lazy evaluation and can
//! represent infinite sequences.

use crate::eval::value::{Value, ThreadSafeEnvironment};
use crate::diagnostics::Result as LambdustResult;
use crate::diagnostics::Error;
use std::sync::{Arc, RwLock};

/// Generator state representing different types of generators
#[derive(Debug, Clone)]
pub enum GeneratorState {
    /// Generator created from a procedure (thunk)
    Procedure {
        /// The procedure to call for next value
        thunk: Value,
        /// Environment for procedure execution
        environment: Arc<ThreadSafeEnvironment>,
    },
    
    /// Generator that yields explicit values
    Values {
        /// Remaining values to yield
        values: Vec<Value>,
        /// Current index in values
        index: usize,
    },
    
    /// Range generator for arithmetic sequences
    Range {
        /// Current value
        current: f64,
        /// Step size
        step: f64,
        /// End value (exclusive)
        end: Option<f64>,
    },
    
    /// Iota generator for counting sequences
    Iota {
        /// Current count
        count: i64,
        /// Remaining items (None for infinite)
        remaining: Option<usize>,
        /// Step size
        step: i64,
    },
    
    /// List generator that yields from a Scheme list
    List {
        /// Current position in list
        current: Value,
    },
    
    /// Vector generator that yields from a vector
    Vector {
        /// The vector to iterate over
        vector: Arc<RwLock<Vec<Value>>>,
        /// Current index
        index: usize,
    },
    
    /// String generator that yields characters
    String {
        /// The string to iterate over
        string: String,
        /// Current index
        index: usize,
    },
    
    /// Exhausted generator (no more values)
    Exhausted,
}

/// Thread-safe generator wrapper
#[derive(Debug, Clone)]
pub struct Generator {
    /// Generator state protected by RwLock
    state: Arc<RwLock<GeneratorState>>,
    /// Optional name for debugging
    name: Option<String>,
    /// EOF object to return when exhausted
    eof_object: Value,
}

/// Thread-safe generator wrapper (alias for consistency with other containers)
pub type ThreadSafeGenerator = Generator;

impl Generator {
    /// Creates a new generator from a procedure (thunk)
    pub fn from_procedure(thunk: Value, environment: Arc<ThreadSafeEnvironment>) -> Self {
        Self {
            state: Arc::new(RwLock::new(GeneratorState::Procedure { thunk, environment })),
            name: None,
            eof_object: Value::symbol_from_str("*eof-object*"),
        }
    }
    
    /// Creates a new generator from explicit values
    pub fn from_values(values: Vec<Value>) -> Self {
        Self {
            state: Arc::new(RwLock::new(GeneratorState::Values { values, index: 0 })),
            name: None,
            eof_object: Value::symbol_from_str("*eof-object*"),
        }
    }
    
    /// Creates a new range generator
    pub fn range(start: f64, end: Option<f64>, step: f64) -> Self {
        Self {
            state: Arc::new(RwLock::new(GeneratorState::Range {
                current: start,
                step,
                end,
            })),
            name: None,
            eof_object: Value::symbol_from_str("*eof-object*"),
        }
    }
    
    /// Creates a new iota generator
    pub fn iota(count: Option<usize>, start: i64, step: i64) -> Self {
        Self {
            state: Arc::new(RwLock::new(GeneratorState::Iota {
                count: start,
                remaining: count,
                step,
            })),
            name: None,
            eof_object: Value::symbol_from_str("*eof-object*"),
        }
    }
    
    /// Creates a new generator from a Scheme list
    pub fn from_list(list: Value) -> Self {
        Self {
            state: Arc::new(RwLock::new(GeneratorState::List { current: list })),
            name: None,
            eof_object: Value::symbol_from_str("*eof-object*"),
        }
    }
    
    /// Creates a new generator from a vector
    pub fn from_vector(vector: Arc<RwLock<Vec<Value>>>) -> Self {
        Self {
            state: Arc::new(RwLock::new(GeneratorState::Vector { vector, index: 0 })),
            name: None,
            eof_object: Value::symbol_from_str("*eof-object*"),
        }
    }
    
    /// Creates a new generator from a string
    pub fn from_string(string: String) -> Self {
        Self {
            state: Arc::new(RwLock::new(GeneratorState::String { string, index: 0 })),
            name: None,
            eof_object: Value::symbol_from_str("*eof-object*"),
        }
    }
    
    /// Creates an already exhausted generator
    pub fn exhausted() -> Self {
        Self {
            state: Arc::new(RwLock::new(GeneratorState::Exhausted)),
            name: None,
            eof_object: Value::symbol_from_str("*eof-object*"),
        }
    }
    
    /// Gets the next value from the generator
    pub fn next(&self) -> LambdustResult<Value> {
        let mut state = self.state.write().map_err(|_| {
            Box::new(Error::RuntimeError {
                message: "Failed to acquire generator lock".to_string(),
                span: None,
            })
        })?;
        
        match &mut *state {
            GeneratorState::Procedure { .. } => {
                // For procedure generators, we would need access to an evaluator
                // This will be implemented when we integrate with the evaluator
                Ok(self.eof_object.clone())
            }
            
            GeneratorState::Values { values, index } => {
                if *index < values.len() {
                    let value = values[*index].clone();
                    *index += 1;
                    Ok(value)
                } else {
                    *state = GeneratorState::Exhausted;
                    Ok(self.eof_object.clone())
                }
            }
            
            GeneratorState::Range { current, step, end } => {
                if let Some(end_val) = end {
                    if (*step > 0.0 && *current >= *end_val) || 
                       (*step < 0.0 && *current <= *end_val) {
                        *state = GeneratorState::Exhausted;
                        return Ok(self.eof_object.clone());
                    }
                }
                
                let value = Value::number(*current);
                *current += *step;
                Ok(value)
            }
            
            GeneratorState::Iota { count, remaining, step } => {
                if let Some(rem) = remaining {
                    if *rem == 0 {
                        *state = GeneratorState::Exhausted;
                        return Ok(self.eof_object.clone());
                    }
                    *rem -= 1;
                }
                
                let value = Value::integer(*count);
                *count += *step;
                Ok(value)
            }
            
            GeneratorState::List { current } => {
                match current {
                    Value::Nil => {
                        *state = GeneratorState::Exhausted;
                        Ok(self.eof_object.clone())
                    }
                    Value::Pair(car, cdr) => {
                        let value = car.as_ref().clone();
                        *current = cdr.as_ref().clone();
                        Ok(value)
                    }
                    Value::MutablePair(car_ref, cdr_ref) => {
                        // We need to get the values before modifying current
                        let car_val = {
                            let car = car_ref.read().map_err(|_| {
                                Box::new(Error::RuntimeError {
                                    message: "Failed to read car".to_string(),
                                    span: None,
                                })
                            })?;
                            car.clone()
                        };
                        let cdr_val = {
                            let cdr = cdr_ref.read().map_err(|_| {
                                Box::new(Error::RuntimeError {
                                    message: "Failed to read cdr".to_string(),
                                    span: None,
                                })
                            })?;
                            cdr.clone()
                        };
                        
                        *current = cdr_val;
                        Ok(car_val)
                    }
                    _ => {
                        // Improper list, yield the final value and exhaust
                        let value = current.clone();
                        *state = GeneratorState::Exhausted;
                        Ok(value)
                    }
                }
            }
            
            GeneratorState::Vector { vector, index } => {
                let (value_opt, should_exhaust) = {
                    let vec_guard = vector.read().map_err(|_| {
                        Box::new(Error::RuntimeError {
                            message: "Failed to read vector".to_string(),
                            span: None,
                        })
                    })?;
                    
                    if *index < vec_guard.len() {
                        let value = vec_guard[*index].clone();
                        (Some(value), false)
                    } else {
                        (None, true)
                    }
                };
                
                if let Some(value) = value_opt {
                    *index += 1;
                    Ok(value)
                } else {
                    *state = GeneratorState::Exhausted;
                    Ok(self.eof_object.clone())
                }
            }
            
            GeneratorState::String { string, index } => {
                let chars: Vec<char> = string.chars().collect();
                if *index < chars.len() {
                    let ch = chars[*index];
                    *index += 1;
                    Ok(Value::Literal(crate::ast::Literal::Character(ch)))
                } else {
                    *state = GeneratorState::Exhausted;
                    Ok(self.eof_object.clone())
                }
            }
            
            GeneratorState::Exhausted => {
                Ok(self.eof_object.clone())
            }
        }
    }
    
    /// Checks if the generator is exhausted
    pub fn is_exhausted(&self) -> bool {
        if let Ok(state) = self.state.read() {
            matches!(*state, GeneratorState::Exhausted)
        } else {
            true // Conservative: if we can't read state, consider exhausted
        }
    }
    
    /// Sets the name of the generator for debugging
    pub fn set_name(&mut self, name: String) {
        self.name = Some(name);
    }
    
    /// Gets the name of the generator
    pub fn name(&self) -> Option<&str> {
        self.name.as_deref()
    }
    
    /// Sets the EOF object returned when the generator is exhausted
    pub fn set_eof_object(&mut self, eof: Value) {
        self.eof_object = eof;
    }
    
    /// Gets the EOF object
    pub fn eof_object(&self) -> &Value {
        &self.eof_object
    }
}

// Thread safety markers - Generator is safe because all its internal state
// is protected by Arc<RwLock<T>> and the EOF object is immutable once set
unsafe impl Send for Generator {}
unsafe impl Sync for Generator {}

impl std::fmt::Display for Generator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(name) = &self.name {
            write!(f, "#<generator:{name}>")
        } else {
            write!(f, "#<generator>")
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::eval::value::ThreadSafeEnvironment;
    
    #[test]
    fn test_values_generator() {
        let values = vec![
            Value::integer(1),
            Value::integer(2),
            Value::integer(3),
        ];
        let generator = Generator::from_values(values);
        
        assert_eq!(generator.next().unwrap(), Value::integer(1));
        assert_eq!(generator.next().unwrap(), Value::integer(2));
        assert_eq!(generator.next().unwrap(), Value::integer(3));
        
        // Should be exhausted now
        assert!(generator.is_exhausted());
        assert_eq!(generator.next().unwrap(), generator.eof_object().clone());
    }
    
    #[test]
    fn test_range_generator() {
        let generator = Generator::range(0.0, Some(3.0), 1.0);
        
        assert_eq!(generator.next().unwrap(), Value::number(0.0));
        assert_eq!(generator.next().unwrap(), Value::number(1.0));
        assert_eq!(generator.next().unwrap(), Value::number(2.0));
        
        // Should be exhausted now (3.0 is exclusive)
        assert!(generator.is_exhausted());
    }
    
    #[test]
    fn test_iota_generator() {
        let generator = Generator::iota(Some(3), 5, 2);
        
        assert_eq!(generator.next().unwrap(), Value::integer(5));
        assert_eq!(generator.next().unwrap(), Value::integer(7));
        assert_eq!(generator.next().unwrap(), Value::integer(9));
        
        // Should be exhausted now
        assert!(generator.is_exhausted());
    }
    
    #[test]
    fn test_list_generator() {
        let list = Value::list(vec![
            Value::integer(1),
            Value::integer(2),
            Value::integer(3),
        ]);
        let generator = Generator::from_list(list);
        
        assert_eq!(generator.next().unwrap(), Value::integer(1));
        assert_eq!(generator.next().unwrap(), Value::integer(2));
        assert_eq!(generator.next().unwrap(), Value::integer(3));
        
        // Should be exhausted now
        assert!(generator.is_exhausted());
    }
    
    #[test]
    fn test_string_generator() {
        let generator = Generator::from_string("abc".to_string());
        
        assert_eq!(generator.next().unwrap(), Value::Literal(crate::ast::Literal::Character('a')));
        assert_eq!(generator.next().unwrap(), Value::Literal(crate::ast::Literal::Character('b')));
        assert_eq!(generator.next().unwrap(), Value::Literal(crate::ast::Literal::Character('c')));
        
        // Should be exhausted now
        assert!(generator.is_exhausted());
    }
    
    #[test]
    fn test_exhausted_generator() {
        let generator = Generator::exhausted();
        
        assert!(generator.is_exhausted());
        assert_eq!(generator.next().unwrap(), generator.eof_object().clone());
    }
}