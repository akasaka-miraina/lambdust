//! SRFI-37: args-fold - A program argument processor
//!
//! This module implements SRFI-37, which provides `args-fold`, a higher-order procedure
//! for processing command-line arguments with support for:
//! - Short options (-h, -v)  
//! - Long options (--help, --verbose)
//! - Options with required/optional values
//! - Unrecognized option handling
//! - Operand (non-option argument) processing

use crate::ast::Literal;
use crate::diagnostics::{Error, Result, Span};
use crate::eval::value::{PrimitiveImpl, PrimitiveProcedure, ThreadSafeEnvironment, Value};
use crate::effects::Effect;
use std::collections::HashMap;
use std::sync::Arc;

/// Represents an option descriptor for args-fold.
#[derive(Debug, Clone)]
pub struct OptionDescriptor {
    /// Option names (both short and long forms)
    pub names: Vec<String>,
    /// Does this option require an argument?
    pub required_arg: bool,
    /// Does this option accept an optional argument?
    pub optional_arg: bool,
    /// Processor procedure for this option
    pub processor: Value,
}

/// Internal state for argument processing
#[derive(Debug)]
struct ArgsProcessor {
    /// Arguments being processed
    args: Vec<String>,
    /// Current position in argument list
    position: usize,
    /// Map of option names to descriptors for fast lookup
    option_map: HashMap<String, OptionDescriptor>,
    /// Unrecognized option handler
    unrecognized_proc: Value,
    /// Operand handler  
    operand_proc: Value,
    /// Current seed values
    seeds: Vec<Value>,
}

impl ArgsProcessor {
    /// Create a new argument processor
    fn new(
        args: Vec<String>,
        options: Vec<OptionDescriptor>,
        unrecognized_proc: Value,
        operand_proc: Value,
        seeds: Vec<Value>,
    ) -> Self {
        // Build option name -> descriptor map for O(1) lookup
        let mut option_map = HashMap::new();
        for descriptor in options {
            for name in &descriptor.names {
                option_map.insert(name.clone(), descriptor.clone());
            }
        }

        Self {
            args,
            position: 0,
            option_map,
            unrecognized_proc,
            operand_proc,
            seeds,
        }
    }

    /// Check if we have more arguments to process
    fn has_next(&self) -> bool {
        self.position < self.args.len()
    }

    /// Get current argument without consuming it
    fn peek(&self) -> Option<&String> {
        self.args.get(self.position)
    }

    /// Consume and return the current argument
    fn next(&mut self) -> Option<String> {
        if self.has_next() {
            let arg = self.args[self.position].clone();
            self.position += 1;
            Some(arg)
        } else {
            None
        }
    }

    /// Process a short option (starts with single dash)
    fn process_short_option(&mut self, arg: &str) -> Result<()> {
        // Handle clustered short options like -abc
        let chars: Vec<char> = arg.chars().skip(1).collect(); // Skip the '-'
        
        for (i, ch) in chars.iter().enumerate() {
            let option_name = ch.to_string();
            
            if let Some(descriptor) = self.option_map.get(&option_name).cloned() {
                let option_value = self.create_option_value(&descriptor)?;
                
                // Determine argument handling
                let argument = if descriptor.required_arg || descriptor.optional_arg {
                    if i == chars.len() - 1 {
                        // Last option in cluster, can take next arg
                        if descriptor.required_arg {
                            self.next().ok_or_else(|| Box::new(Error::runtime_error(
                                format!("Option -{} requires an argument", ch),
                                None
                            )))?
                        } else {
                            // Optional argument - take next if it doesn't look like option
                            if let Some(next_arg) = self.peek() {
                                if !next_arg.starts_with('-') {
                                    self.next().unwrap()
                                } else {
                                    String::new() // Empty string for no optional arg
                                }
                            } else {
                                String::new()
                            }
                        }
                    } else if descriptor.required_arg {
                        return Err(Box::new(Error::runtime_error(
                            format!("Option -{} in cluster requires an argument", ch),
                            None
                        )));
                    } else {
                        String::new() // Optional arg not available in cluster
                    }
                } else {
                    String::new()
                };

                let arg_value = if argument.is_empty() { 
                    Value::boolean(false) 
                } else { 
                    Value::string(argument) 
                };

                // Call processor: (option name arg . seeds) -> seeds
                self.seeds = self.call_processor(
                    &descriptor.processor,
                    vec![
                        option_value,
                        Value::string(option_name),
                        arg_value,
                    ],
                )?;
            } else {
                // Unrecognized short option
                self.seeds = self.call_processor(
                    &self.unrecognized_proc.clone(),
                    vec![
                        Value::string(format!("-{}", ch)),
                        Value::string(ch.to_string()),
                        Value::boolean(false),
                    ],
                )?;
            }
        }
        
        Ok(())
    }

    /// Process a long option (starts with double dash)
    fn process_long_option(&mut self, arg: &str) -> Result<()> {
        let (option_name, embedded_arg) = if let Some(eq_pos) = arg.find('=') {
            (arg[2..eq_pos].to_string(), Some(arg[eq_pos + 1..].to_string()))
        } else {
            (arg[2..].to_string(), None)
        };

        if let Some(descriptor) = self.option_map.get(&option_name).cloned() {
            let option_value = self.create_option_value(&descriptor)?;
            
            let argument = if let Some(embedded) = embedded_arg {
                embedded
            } else if descriptor.required_arg {
                self.next().ok_or_else(|| Box::new(Error::runtime_error(
                    format!("Option --{} requires an argument", option_name),
                    None
                )))?
            } else if descriptor.optional_arg {
                if let Some(next_arg) = self.peek() {
                    if !next_arg.starts_with('-') {
                        self.next().unwrap()
                    } else {
                        String::new()
                    }
                } else {
                    String::new()
                }
            } else {
                String::new()
            };

            let arg_value = if argument.is_empty() { 
                Value::boolean(false) 
            } else { 
                Value::string(argument) 
            };

            self.seeds = self.call_processor(
                &descriptor.processor,
                vec![
                    option_value,
                    Value::string(option_name),
                    arg_value,
                ],
            )?;
        } else {
            // Unrecognized long option
            let arg_value = if let Some(embedded) = embedded_arg {
                Value::string(embedded)
            } else {
                Value::boolean(false)
            };

            self.seeds = self.call_processor(
                &self.unrecognized_proc.clone(),
                vec![
                    Value::string(arg.to_string()),
                    Value::string(option_name),
                    arg_value,
                ],
            )?;
        }

        Ok(())
    }

    /// Process an operand (non-option argument)
    fn process_operand(&mut self, operand: String) -> Result<()> {
        self.seeds = self.call_processor(
            &self.operand_proc.clone(),
            vec![Value::string(operand)],
        )?;
        Ok(())
    }

    /// Create a Value representation of an option descriptor
    fn create_option_value(&self, descriptor: &OptionDescriptor) -> Result<Value> {
        // Create a list: (option names required-arg? optional-arg? processor)
        let names_list = Value::list(
            descriptor.names.iter()
                .map(|name| {
                    if name.len() == 1 {
                        // Short option - represent as character
                        Value::Literal(Literal::Character(name.chars().next().unwrap()))
                    } else {
                        // Long option - represent as string
                        Value::string(name.clone())
                    }
                })
                .collect()
        );

        Ok(Value::list(vec![
            Value::symbol_from_str("option"),
            names_list,
            Value::boolean(descriptor.required_arg),
            Value::boolean(descriptor.optional_arg),
            descriptor.processor.clone(),
        ]))
    }

    /// Call a processor procedure with arguments and current seeds
    fn call_processor(&self, proc: &Value, mut args: Vec<Value>) -> Result<Vec<Value>> {
        // Add current seeds to argument list
        args.extend(self.seeds.iter().cloned());
        
        match proc {
            Value::Primitive(prim) => {
                match &prim.implementation {
                    PrimitiveImpl::Native(func) => {
                        let result = func(&args)?;
                        Ok(vec![result])
                    }
                    PrimitiveImpl::RustFn(func) => {
                        let result = func(&args)?;
                        Ok(vec![result])
                    }
                    _ => Err(Box::new(Error::runtime_error(
                        "Expected native procedure for args-fold processor",
                        None
                    )))
                }
            }
            Value::Procedure(_) => {
                // TODO: Implement procedure call through evaluator
                // For now, return unchanged seeds
                Ok(self.seeds.clone())
            }
            _ => Err(Box::new(Error::runtime_error(
                "Expected procedure for args-fold processor",
                None
            )))
        }
    }

    /// Main processing loop
    fn process(&mut self) -> Result<Vec<Value>> {
        while self.has_next() {
            let arg = self.next().unwrap();
            
            if arg == "--" {
                // End of options marker - rest are operands
                while self.has_next() {
                    let operand = self.next().unwrap();
                    self.process_operand(operand)?;
                }
                break;
            } else if arg.starts_with("--") && arg.len() > 2 {
                // Long option
                self.process_long_option(&arg)?;
            } else if arg.starts_with('-') && arg.len() > 1 {
                // Short option(s)
                self.process_short_option(&arg)?;
            } else {
                // Operand
                self.process_operand(arg)?;
            }
        }

        Ok(self.seeds.clone())
    }
}

/// Implementation of `(option names required-arg? optional-arg? processor)`
/// Creates an option descriptor.
pub fn option(args: &[Value]) -> Result<Value> {
    if args.len() != 4 {
        return Err(Box::new(Error::runtime_error(
            format!("option: expected 4 arguments, got {}", args.len()),
            None
        )));
    }

    // Extract names list
    let names = if args[0].is_list() {
        let mut names = Vec::new();
        let mut current = &args[0];
        while let Value::Pair(car, cdr) = current {
            match car.as_ref() {
                Value::Literal(Literal::Character(ch)) => names.push(ch.to_string()),
                Value::Literal(Literal::String(s)) => names.push(s.to_string()),
                Value::Symbol(_) => {
                    // For now, just use a placeholder - symbol to string conversion is complex
                    names.push("symbol".to_string());
                },
                _ => return Err(Box::new(Error::runtime_error(
                    "option: names must be characters or strings",
                    None
                )))
            }
            current = cdr.as_ref();
        }
        names
    } else {
        return Err(Box::new(Error::runtime_error(
            "option: first argument must be a list of names",
            None
        )));
    };

    let required_arg = match &args[1] {
        Value::Literal(Literal::Boolean(b)) => *b,
        _ => return Err(Box::new(Error::runtime_error(
            "option: required-arg? must be a boolean",
            None
        )))
    };

    let optional_arg = match &args[2] {
        Value::Literal(Literal::Boolean(b)) => *b,
        _ => return Err(Box::new(Error::runtime_error(
            "option: optional-arg? must be a boolean", 
            None
        )))
    };

    if required_arg && optional_arg {
        return Err(Box::new(Error::runtime_error(
            "option: cannot have both required and optional arguments",
            None
        )));
    }

    let processor = args[3].clone();

    // Create option descriptor as a special internal value
    // For now, represent as a tagged list
    Ok(Value::list(vec![
        Value::symbol_from_str("option-descriptor"),
        Value::list(names.into_iter().map(Value::string).collect()),
        Value::boolean(required_arg),
        Value::boolean(optional_arg),
        processor,
    ]))
}

/// Implementation of `(args-fold args options unrecognized-option-proc operand-proc . seeds)`
/// Main args-fold procedure.
pub fn args_fold(args: &[Value]) -> Result<Value> {
    if args.len() < 4 {
        return Err(Box::new(Error::runtime_error(
            format!("args-fold: expected at least 4 arguments, got {}", args.len()),
            None
        )));
    }

    // Extract arguments list
    let arg_strings = if args[0].is_list() {
        let mut strings = Vec::new();
        let mut current = &args[0];
        while let Value::Pair(car, cdr) = current {
            match car.as_ref() {
                Value::Literal(Literal::String(s)) => strings.push(s.to_string()),
                _ => return Err(Box::new(Error::runtime_error(
                    "args-fold: args must be a list of strings",
                    None
                )))
            }
            current = cdr.as_ref();
        }
        strings
    } else {
        return Err(Box::new(Error::runtime_error(
            "args-fold: first argument must be a list",
            None
        )));
    };

    // Extract options list
    let option_descriptors = if args[1].is_list() {
        let mut descriptors = Vec::new();
        let mut current = &args[1];
        while let Value::Pair(car, cdr) = current {
            // Each item should be an option descriptor list
            if car.is_list() {
                // Convert car to a vector for easier processing
                let mut desc_items = Vec::new();
                let mut desc_current = car.as_ref();
                while let Value::Pair(desc_car, desc_cdr) = desc_current {
                    desc_items.push(desc_car.as_ref());
                    desc_current = desc_cdr.as_ref();
                }
                
                // Check if this looks like an option descriptor
                if desc_items.len() >= 5 && matches!(desc_items[0], Value::Symbol(_)) {
                    // Extract names from the second element
                    let names = if desc_items[1].is_list() {
                        let mut name_vec = Vec::new();
                        let mut name_current = desc_items[1];
                        while let Value::Pair(name_car, name_cdr) = name_current {
                            if let Value::Literal(Literal::String(s)) = name_car.as_ref() {
                                name_vec.push(s.to_string());
                            }
                            name_current = name_cdr.as_ref();
                        }
                        name_vec
                    } else {
                        Vec::new()
                    };

                    let required_arg = matches!(desc_items[2], Value::Literal(Literal::Boolean(true)));
                    let optional_arg = matches!(desc_items[3], Value::Literal(Literal::Boolean(true)));
                    let processor = desc_items[4].clone();

                    descriptors.push(OptionDescriptor {
                        names,
                        required_arg,
                        optional_arg,
                        processor,
                    });
                }
            }
            current = cdr.as_ref();
        }
        descriptors
    } else {
        return Err(Box::new(Error::runtime_error(
            "args-fold: second argument must be a list of options",
            None
        )));
    };

    let unrecognized_proc = args[2].clone();
    let operand_proc = args[3].clone();
    let seeds = args[4..].to_vec();

    // Create processor and run it
    let mut processor = ArgsProcessor::new(
        arg_strings,
        option_descriptors,
        unrecognized_proc,
        operand_proc,
        seeds,
    );

    let final_seeds = processor.process()?;

    // Return the final seed values
    match final_seeds.len() {
        0 => Ok(Value::Nil),
        1 => Ok(final_seeds[0].clone()),
        _ => Ok(Value::list(final_seeds)),
    }
}

/// Bind SRFI-37 procedures to the environment
pub fn bind_srfi37_procedures(env: &Arc<ThreadSafeEnvironment>) {
    // Helper function to bind a primitive procedure
    fn bind_primitive(
        env: &Arc<ThreadSafeEnvironment>,
        name: &str,
        arity_min: usize,
        arity_max: Option<usize>,
        implementation: fn(&[Value]) -> Result<Value>,
        effects: Vec<Effect>,
    ) {
        env.define(
            name.to_string(),
            Value::Primitive(Arc::new(PrimitiveProcedure {
                name: name.to_string(),
                arity_min,
                arity_max,
                implementation: PrimitiveImpl::Native(implementation),
                effects,
            })),
        );
    }

    // Bind core SRFI-37 procedures
    bind_primitive(
        env,
        "option",
        4,
        Some(4),
        option,
        vec![Effect::Pure],
    );

    bind_primitive(
        env,
        "args-fold",
        4,
        None, // Variable arity for multiple seeds
        args_fold,
        vec![Effect::Pure],
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::eval::value::ThreadSafeEnvironment;
    use std::sync::Arc;

    #[test]
    fn test_option_creation() {
        let names = Value::list(vec![
            Value::Literal(Literal::Character('h')),
            Value::string("help"),
        ]);
        
        let args = [
            names,
            Value::boolean(false),
            Value::boolean(false),
            Value::Primitive(Arc::new(PrimitiveProcedure {
                name: "test-proc".to_string(),
                arity_min: 0,
                arity_max: None,
                implementation: PrimitiveImpl::Native(|_| Ok(Value::Nil)),
                effects: vec![],
            })),
        ];

        let result = option(&args);
        assert!(result.is_ok());
        
        // Check that result is a list structure
        if let Ok(result_value) = result {
            assert!(result_value.is_list());
        }
    }

    #[test]
    fn test_option_invalid_args() {
        // Test with wrong number of arguments
        let result = option(&[]);
        assert!(result.is_err());

        // Test with both required and optional args
        let names = Value::list(vec![Value::Literal(Literal::Character('h'))]);
        let args = [
            names,
            Value::boolean(true),  // required
            Value::boolean(true),  // optional - should be error
            Value::Nil,
        ];
        let result = option(&args);
        assert!(result.is_err());
    }
}