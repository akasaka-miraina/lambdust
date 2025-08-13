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

/// Type alias for a procedure evaluator callback function.
/// Takes a procedure (thunk) and returns its evaluated result.
pub type ProcedureEvaluator = Box<dyn Fn(&Value) -> LambdustResult<Value> + Send + Sync>;

/// Container for optional procedure evaluator
#[derive(Clone)]
pub struct EvaluatorRef {
    evaluator: Option<Arc<ProcedureEvaluator>>,
}

impl std::fmt::Debug for EvaluatorRef {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.evaluator.is_some() {
            write!(f, "EvaluatorRef(Some(<evaluator>))")
        } else {
            write!(f, "EvaluatorRef(None)")
        }
    }
}

impl Default for EvaluatorRef {
    fn default() -> Self {
        Self::new()
    }
}

impl EvaluatorRef {
    /// Creates a new empty evaluator reference.
    pub fn new() -> Self {
        Self { evaluator: None }
    }
    
    /// Creates a new evaluator reference with a procedure evaluator.
    pub fn with_evaluator(evaluator: Arc<ProcedureEvaluator>) -> Self {
        Self { 
            evaluator: Some(evaluator)
        }
    }
    
    /// Evaluates a procedure using the stored evaluator.
    pub fn evaluate(&self, procedure: &Value) -> LambdustResult<Value> {
        if let Some(eval) = &self.evaluator {
            eval(procedure)
        } else {
            // Return EOF object if no evaluator is available
            Ok(Value::symbol_from_str("*eof-object*"))
        }
    }
}

/// Generator state representing different types of generators
#[derive(Debug, Clone)]
pub enum GeneratorState {
    /// Generator created from a procedure (thunk)
    Procedure {
        /// The procedure to call for next value
        thunk: Value,
        /// Environment for procedure execution
        environment: Arc<ThreadSafeEnvironment>,
        /// Optional evaluator for calling the procedure
        evaluator: EvaluatorRef,
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
    
    /// Unfold generator that generates values using predicates and transformers
    Unfold {
        /// Stop predicate function (when to stop)
        stop_predicate: Value,
        /// Mapper function (how to transform seed to value)
        mapper: Value,
        /// Successor function (how to generate next seed)
        successor: Value,
        /// Current seed value
        seed: Value,
        /// Optional evaluator for calling procedures
        evaluator: EvaluatorRef,
    },
    
    /// Tabulate generator that generates values using an index function
    Tabulate {
        /// Function that maps index to value
        func: Value,
        /// Current index
        index: usize,
        /// Maximum count (None for infinite)
        max_count: Option<usize>,
        /// Optional evaluator for calling procedures  
        evaluator: EvaluatorRef,
    },
    
    /// Mapped generator that applies a function to each value from source
    Map {
        /// Source generator to map over
        source: Arc<Generator>,
        /// Mapping function
        mapper: Value,
        /// Optional evaluator for calling procedures
        evaluator: EvaluatorRef,
    },
    
    /// Filtered generator that only yields values that satisfy a predicate
    Filter {
        /// Source generator to filter
        source: Arc<Generator>,
        /// Predicate function
        predicate: Value,
        /// Optional evaluator for calling procedures
        evaluator: EvaluatorRef,
    },
    
    /// Take generator that yields up to n values from source
    Take {
        /// Source generator to take from
        source: Arc<Generator>,
        /// Number of values to take
        count: usize,
        /// Number of values taken so far
        taken: usize,
    },
    
    /// Drop generator that skips the first n values from source
    Drop {
        /// Source generator to drop from
        source: Arc<Generator>,
        /// Number of values to drop
        count: usize,
        /// Whether we've performed the drop yet
        dropped: bool,
    },
    
    /// Append generator that yields from first, then second generator
    Append {
        /// First generator to consume
        first: Arc<Generator>,
        /// Second generator to consume after first is exhausted
        second: Arc<Generator>,
        /// Whether we've switched to the second generator
        using_second: bool,
    },
    
    /// Concatenate generator that yields from a list of generators in sequence
    Concatenate {
        /// Remaining generators to consume
        generators: Vec<Arc<Generator>>,
        /// Current generator index
        current_index: usize,
    },
    
    /// Zip generator that yields tuples from multiple generators
    Zip {
        /// Generators to zip together
        sources: Vec<Arc<Generator>>,
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
            state: Arc::new(RwLock::new(GeneratorState::Procedure { 
                thunk, 
                environment, 
                evaluator: EvaluatorRef::new()
            })),
            name: None,
            eof_object: Value::symbol_from_str("*eof-object*"),
        }
    }
    
    /// Creates a new generator from a procedure (thunk) with an evaluator
    pub fn from_procedure_with_evaluator(
        thunk: Value, 
        environment: Arc<ThreadSafeEnvironment>,
        evaluator: Arc<ProcedureEvaluator>
    ) -> Self {
        Self {
            state: Arc::new(RwLock::new(GeneratorState::Procedure { 
                thunk, 
                environment, 
                evaluator: EvaluatorRef::with_evaluator(evaluator)
            })),
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
    
    /// Creates a new unfold generator
    pub fn unfold(
        stop_predicate: Value,
        mapper: Value,
        successor: Value,
        seed: Value,
    ) -> Self {
        Self {
            state: Arc::new(RwLock::new(GeneratorState::Unfold {
                stop_predicate,
                mapper,
                successor,
                seed,
                evaluator: EvaluatorRef::new(),
            })),
            name: None,
            eof_object: Value::symbol_from_str("*eof-object*"),
        }
    }
    
    /// Creates a new unfold generator with an evaluator
    pub fn unfold_with_evaluator(
        stop_predicate: Value,
        mapper: Value,
        successor: Value,
        seed: Value,
        evaluator: Arc<ProcedureEvaluator>,
    ) -> Self {
        Self {
            state: Arc::new(RwLock::new(GeneratorState::Unfold {
                stop_predicate,
                mapper,
                successor,
                seed,
                evaluator: EvaluatorRef::with_evaluator(evaluator),
            })),
            name: None,
            eof_object: Value::symbol_from_str("*eof-object*"),
        }
    }
    
    /// Creates a new tabulate generator
    pub fn tabulate(func: Value, max_count: Option<usize>) -> Self {
        Self {
            state: Arc::new(RwLock::new(GeneratorState::Tabulate {
                func,
                index: 0,
                max_count,
                evaluator: EvaluatorRef::new(),
            })),
            name: None,
            eof_object: Value::symbol_from_str("*eof-object*"),
        }
    }
    
    /// Creates a new tabulate generator with an evaluator
    pub fn tabulate_with_evaluator(
        func: Value,
        max_count: Option<usize>,
        evaluator: Arc<ProcedureEvaluator>,
    ) -> Self {
        Self {
            state: Arc::new(RwLock::new(GeneratorState::Tabulate {
                func,
                index: 0,
                max_count,
                evaluator: EvaluatorRef::with_evaluator(evaluator),
            })),
            name: None,
            eof_object: Value::symbol_from_str("*eof-object*"),
        }
    }
    
    /// Creates a new map generator
    pub fn map(source: Arc<Generator>, mapper: Value) -> Self {
        Self {
            state: Arc::new(RwLock::new(GeneratorState::Map {
                source,
                mapper,
                evaluator: EvaluatorRef::new(),
            })),
            name: None,
            eof_object: Value::symbol_from_str("*eof-object*"),
        }
    }
    
    /// Creates a new map generator with an evaluator
    pub fn map_with_evaluator(
        source: Arc<Generator>,
        mapper: Value,
        evaluator: Arc<ProcedureEvaluator>,
    ) -> Self {
        Self {
            state: Arc::new(RwLock::new(GeneratorState::Map {
                source,
                mapper,
                evaluator: EvaluatorRef::with_evaluator(evaluator),
            })),
            name: None,
            eof_object: Value::symbol_from_str("*eof-object*"),
        }
    }
    
    /// Creates a new filter generator
    pub fn filter(source: Arc<Generator>, predicate: Value) -> Self {
        Self {
            state: Arc::new(RwLock::new(GeneratorState::Filter {
                source,
                predicate,
                evaluator: EvaluatorRef::new(),
            })),
            name: None,
            eof_object: Value::symbol_from_str("*eof-object*"),
        }
    }
    
    /// Creates a new filter generator with an evaluator
    pub fn filter_with_evaluator(
        source: Arc<Generator>,
        predicate: Value,
        evaluator: Arc<ProcedureEvaluator>,
    ) -> Self {
        Self {
            state: Arc::new(RwLock::new(GeneratorState::Filter {
                source,
                predicate,
                evaluator: EvaluatorRef::with_evaluator(evaluator),
            })),
            name: None,
            eof_object: Value::symbol_from_str("*eof-object*"),
        }
    }
    
    /// Creates a new take generator
    pub fn take(source: Arc<Generator>, count: usize) -> Self {
        Self {
            state: Arc::new(RwLock::new(GeneratorState::Take {
                source,
                count,
                taken: 0,
            })),
            name: None,
            eof_object: Value::symbol_from_str("*eof-object*"),
        }
    }
    
    /// Creates a new drop generator
    pub fn drop(source: Arc<Generator>, count: usize) -> Self {
        Self {
            state: Arc::new(RwLock::new(GeneratorState::Drop {
                source,
                count,
                dropped: false,
            })),
            name: None,
            eof_object: Value::symbol_from_str("*eof-object*"),
        }
    }
    
    /// Creates a new append generator
    pub fn append(first: Arc<Generator>, second: Arc<Generator>) -> Self {
        Self {
            state: Arc::new(RwLock::new(GeneratorState::Append {
                first,
                second,
                using_second: false,
            })),
            name: None,
            eof_object: Value::symbol_from_str("*eof-object*"),
        }
    }
    
    /// Creates a new concatenate generator
    pub fn concatenate(generators: Vec<Arc<Generator>>) -> Self {
        Self {
            state: Arc::new(RwLock::new(GeneratorState::Concatenate {
                generators,
                current_index: 0,
            })),
            name: None,
            eof_object: Value::symbol_from_str("*eof-object*"),
        }
    }
    
    /// Creates a new zip generator
    pub fn zip(sources: Vec<Arc<Generator>>) -> Self {
        Self {
            state: Arc::new(RwLock::new(GeneratorState::Zip { sources })),
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
            GeneratorState::Procedure { thunk, evaluator, .. } => {
                // Call the thunk using the evaluator if available
                match evaluator.evaluate(thunk) {
                    Ok(value) => {
                        // Check if the generator returned the EOF object
                        if value == self.eof_object {
                            *state = GeneratorState::Exhausted;
                        }
                        Ok(value)
                    }
                    Err(_) => {
                        // On error, mark generator as exhausted
                        *state = GeneratorState::Exhausted;
                        Ok(self.eof_object.clone())
                    }
                }
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
            
            GeneratorState::Unfold { stop_predicate, mapper, successor, seed, evaluator } => {
                // First check if we should stop
                let should_stop = match evaluator.evaluate(&Value::list(vec![stop_predicate.clone(), seed.clone()])) {
                    Ok(value) => value.is_truthy(),
                    Err(_) => true, // On error, stop generation
                };
                
                if should_stop {
                    *state = GeneratorState::Exhausted;
                    return Ok(self.eof_object.clone());
                }
                
                // Map the current seed to a value
                let current_value = match evaluator.evaluate(&Value::list(vec![mapper.clone(), seed.clone()])) {
                    Ok(value) => value,
                    Err(_) => {
                        *state = GeneratorState::Exhausted;
                        return Ok(self.eof_object.clone());
                    }
                };
                
                // Generate the next seed
                let next_seed = match evaluator.evaluate(&Value::list(vec![successor.clone(), seed.clone()])) {
                    Ok(new_seed) => new_seed,
                    Err(_) => {
                        *state = GeneratorState::Exhausted;
                        return Ok(self.eof_object.clone());
                    }
                };
                
                *seed = next_seed;
                Ok(current_value)
            }
            
            GeneratorState::Tabulate { func, index, max_count, evaluator } => {
                // Check if we've reached the maximum count
                if let Some(max) = max_count {
                    if *index >= *max {
                        *state = GeneratorState::Exhausted;
                        return Ok(self.eof_object.clone());
                    }
                }
                
                // Call the function with the current index
                let current_value = match evaluator.evaluate(&Value::list(vec![func.clone(), Value::integer(*index as i64)])) {
                    Ok(value) => value,
                    Err(_) => {
                        *state = GeneratorState::Exhausted;
                        return Ok(self.eof_object.clone());
                    }
                };
                
                *index += 1;
                Ok(current_value)
            }
            
            GeneratorState::Map { source, mapper, evaluator } => {
                // Get the next value from the source generator
                match source.next() {
                    Ok(value) => {
                        // Check if source is exhausted
                        if value == *source.eof_object() {
                            *state = GeneratorState::Exhausted;
                            return Ok(self.eof_object.clone());
                        }
                        
                        // Apply the mapper function
                        match evaluator.evaluate(&Value::list(vec![mapper.clone(), value])) {
                            Ok(mapped_value) => Ok(mapped_value),
                            Err(_) => {
                                *state = GeneratorState::Exhausted;
                                Ok(self.eof_object.clone())
                            }
                        }
                    }
                    Err(_) => {
                        *state = GeneratorState::Exhausted;
                        Ok(self.eof_object.clone())
                    }
                }
            }
            
            GeneratorState::Filter { source, predicate, evaluator } => {
                // Keep trying until we find a value that satisfies the predicate
                loop {
                    match source.next() {
                        Ok(value) => {
                            // Check if source is exhausted
                            if value == *source.eof_object() {
                                *state = GeneratorState::Exhausted;
                                return Ok(self.eof_object.clone());
                            }
                            
                            // Test the predicate
                            match evaluator.evaluate(&Value::list(vec![predicate.clone(), value.clone()])) {
                                Ok(result) => {
                                    if result.is_truthy() {
                                        return Ok(value);
                                    }
                                    // Continue loop if predicate is false
                                }
                                Err(_) => {
                                    *state = GeneratorState::Exhausted;
                                    return Ok(self.eof_object.clone());
                                }
                            }
                        }
                        Err(_) => {
                            *state = GeneratorState::Exhausted;
                            return Ok(self.eof_object.clone());
                        }
                    }
                }
            }
            
            GeneratorState::Take { source, count, taken } => {
                // Check if we've taken enough values
                if *taken >= *count {
                    *state = GeneratorState::Exhausted;
                    return Ok(self.eof_object.clone());
                }
                
                // Get the next value from source
                match source.next() {
                    Ok(value) => {
                        // Check if source is exhausted
                        if value == *source.eof_object() {
                            *state = GeneratorState::Exhausted;
                            return Ok(self.eof_object.clone());
                        }
                        
                        *taken += 1;
                        Ok(value)
                    }
                    Err(_) => {
                        *state = GeneratorState::Exhausted;
                        Ok(self.eof_object.clone())
                    }
                }
            }
            
            GeneratorState::Drop { source, count, dropped } => {
                // Perform the drop if we haven't yet
                if !*dropped {
                    for _ in 0..*count {
                        match source.next() {
                            Ok(value) => {
                                // If source is exhausted during drop, we're done
                                if value == *source.eof_object() {
                                    *state = GeneratorState::Exhausted;
                                    return Ok(self.eof_object.clone());
                                }
                            }
                            Err(_) => {
                                *state = GeneratorState::Exhausted;
                                return Ok(self.eof_object.clone());
                            }
                        }
                    }
                    *dropped = true;
                }
                
                // Now just forward values from the source
                match source.next() {
                    Ok(value) => {
                        if value == *source.eof_object() {
                            *state = GeneratorState::Exhausted;
                        }
                        Ok(value)
                    }
                    Err(_) => {
                        *state = GeneratorState::Exhausted;
                        Ok(self.eof_object.clone())
                    }
                }
            }
            
            GeneratorState::Append { first, second, using_second } => {
                if !*using_second {
                    // Try to get value from first generator
                    match first.next() {
                        Ok(value) => {
                            if value == *first.eof_object() {
                                // First generator is exhausted, switch to second
                                *using_second = true;
                                // Recursively call to get value from second generator
                                self.next()
                            } else {
                                Ok(value)
                            }
                        }
                        Err(_) => {
                            // Error in first generator, switch to second
                            *using_second = true;
                            self.next()
                        }
                    }
                } else {
                    // Use second generator
                    match second.next() {
                        Ok(value) => {
                            if value == *second.eof_object() {
                                *state = GeneratorState::Exhausted;
                            }
                            Ok(value)
                        }
                        Err(_) => {
                            *state = GeneratorState::Exhausted;
                            Ok(self.eof_object.clone())
                        }
                    }
                }
            }
            
            GeneratorState::Concatenate { generators, current_index } => {
                // Find a generator that can produce a value
                while *current_index < generators.len() {
                    let current_gen = &generators[*current_index];
                    match current_gen.next() {
                        Ok(value) => {
                            if value == *current_gen.eof_object() {
                                // Current generator is exhausted, move to next
                                *current_index += 1;
                                continue;
                            } else {
                                return Ok(value);
                            }
                        }
                        Err(_) => {
                            // Error in current generator, move to next
                            *current_index += 1;
                            continue;
                        }
                    }
                }
                
                // All generators are exhausted
                *state = GeneratorState::Exhausted;
                Ok(self.eof_object.clone())
            }
            
            GeneratorState::Zip { sources } => {
                if sources.is_empty() {
                    *state = GeneratorState::Exhausted;
                    return Ok(self.eof_object.clone());
                }
                
                let mut values = Vec::new();
                
                // Get one value from each source generator
                for source in sources {
                    match source.next() {
                        Ok(value) => {
                            if value == *source.eof_object() {
                                // Any generator being exhausted exhausts the zip
                                *state = GeneratorState::Exhausted;
                                return Ok(self.eof_object.clone());
                            }
                            values.push(value);
                        }
                        Err(_) => {
                            // Error in any generator exhausts the zip
                            *state = GeneratorState::Exhausted;
                            return Ok(self.eof_object.clone());
                        }
                    }
                }
                
                // Return the values as a list (tuple representation in Scheme)
                Ok(Value::list(values))
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