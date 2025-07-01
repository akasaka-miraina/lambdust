//! Example demonstrating the Lambdust bridge API

use lambdust::{LambdustBridge, Result, Value};
use std::collections::HashMap;

// Example external struct that we want to expose to Scheme
#[derive(Debug, Clone)]
struct Calculator {
    memory: f64,
}

impl Calculator {
    fn new() -> Self {
        Calculator { memory: 0.0 }
    }

    fn add(&mut self, value: f64) -> f64 {
        self.memory += value;
        self.memory
    }

    fn multiply(&mut self, value: f64) -> f64 {
        self.memory *= value;
        self.memory
    }

    fn clear(&mut self) -> f64 {
        self.memory = 0.0;
        self.memory
    }

    fn get_memory(&self) -> f64 {
        self.memory
    }
}

// Example user data structure
#[derive(Debug, Clone)]
struct User {
    name: String,
    age: u32,
    email: String,
}

impl User {
    fn new(name: String, age: u32, email: String) -> Self {
        User { name, age, email }
    }

    fn get_name(&self) -> &str {
        &self.name
    }

    fn get_age(&self) -> u32 {
        self.age
    }

    fn set_age(&mut self, age: u32) {
        self.age = age;
    }
}

fn main() -> Result<()> {
    let mut bridge = LambdustBridge::new();

    // Register simple mathematical functions
    bridge.register_function("square", Some(1), |args| {
        let n = match &args[0] {
            Value::Number(num) => match num {
                lambdust::lexer::SchemeNumber::Integer(i) => *i as f64,
                lambdust::lexer::SchemeNumber::Real(r) => *r,
                _ => {
                    return Err(lambdust::error::LambdustError::TypeError(
                        "Expected number".to_string(),
                    ));
                }
            },
            _ => {
                return Err(lambdust::error::LambdustError::TypeError(
                    "Expected number".to_string(),
                ));
            }
        };
        Ok(Value::from(n * n))
    });

    bridge.register_function("factorial", Some(1), |args| {
        let n = match &args[0] {
            Value::Number(num) => match num {
                lambdust::lexer::SchemeNumber::Integer(i) => *i,
                _ => {
                    return Err(lambdust::error::LambdustError::TypeError(
                        "Expected integer".to_string(),
                    ));
                }
            },
            _ => {
                return Err(lambdust::error::LambdustError::TypeError(
                    "Expected integer".to_string(),
                ));
            }
        };

        if n < 0 {
            return Err(lambdust::error::LambdustError::RuntimeError(
                "Factorial of negative number".to_string(),
            ));
        }

        let mut result = 1i64;
        for i in 1..=n {
            result *= i;
        }

        Ok(Value::from(result))
    });

    // Register string manipulation functions
    bridge.register_function("string-upper", Some(1), |args| match &args[0] {
        Value::String(s) => Ok(Value::from(s.to_uppercase())),
        _ => Err(lambdust::error::LambdustError::TypeError(
            "Expected string".to_string(),
        )),
    });

    bridge.register_function("string-reverse", Some(1), |args| match &args[0] {
        Value::String(s) => Ok(Value::from(s.chars().rev().collect::<String>())),
        _ => Err(lambdust::error::LambdustError::TypeError(
            "Expected string".to_string(),
        )),
    });

    // Register a function that creates external objects
    bridge.register_function("make-user", Some(3), |args| {
        let name = match &args[0] {
            Value::String(s) => s.clone(),
            _ => {
                return Err(lambdust::error::LambdustError::TypeError(
                    "Expected string for name".to_string(),
                ));
            }
        };

        let age = match &args[1] {
            Value::Number(num) => match num {
                lambdust::lexer::SchemeNumber::Integer(i) => *i as u32,
                _ => {
                    return Err(lambdust::error::LambdustError::TypeError(
                        "Expected integer for age".to_string(),
                    ));
                }
            },
            _ => {
                return Err(lambdust::error::LambdustError::TypeError(
                    "Expected integer for age".to_string(),
                ));
            }
        };

        let email = match &args[2] {
            Value::String(s) => s.clone(),
            _ => {
                return Err(lambdust::error::LambdustError::TypeError(
                    "Expected string for email".to_string(),
                ));
            }
        };

        let user = User::new(name, age, email);
        // In a real implementation, we'd register this object and return its ID
        Ok(Value::from(format!("User created: {:?}", user)))
    });

    // Define some variables
    bridge.define("pi", Value::from(3.14159265359));
    bridge.define("greeting", Value::from("Hello from Lambdust!"));

    // Test basic arithmetic
    println!("=== Basic Arithmetic ===");
    let result = bridge.eval("(+ 1 2 3)")?;
    println!("(+ 1 2 3) = {}", result);

    let result = bridge.eval("(* pi pi)")?;
    println!("(* pi pi) = {}", result);

    // Test external functions
    println!("\n=== External Functions ===");
    let result = bridge.eval("(call-external \"square\" 5)")?;
    println!("(call-external \"square\" 5) = {}", result);

    let result = bridge.eval("(call-external \"factorial\" 6)")?;
    println!("(call-external \"factorial\" 6) = {}", result);

    let result = bridge.eval("(call-external \"string-upper\" \"hello world\")")?;
    println!(
        "(call-external \"string-upper\" \"hello world\") = {}",
        result
    );

    let result = bridge.eval("(call-external \"string-reverse\" \"lambdust\")")?;
    println!(
        "(call-external \"string-reverse\" \"lambdust\") = {}",
        result
    );

    // Test user creation
    println!("\n=== Object Creation ===");
    let result = bridge.eval("(call-external \"make-user\" \"Alice\" 30 \"alice@example.com\")")?;
    println!("User creation result: {}", result);

    // Test macros
    println!("\n=== Macros ===");
    let result = bridge.eval("(let ((x 10) (y 20)) (+ x y))")?;
    println!("(let ((x 10) (y 20)) (+ x y)) = {}", result);

    let result = bridge.eval("(cond ((< 5 3) 'less) ((> 5 3) 'greater) (else 'equal))")?;
    println!(
        "(cond ((< 5 3) 'less) ((> 5 3) 'greater) (else 'equal)) = {}",
        result
    );

    // Test function definition
    println!("\n=== Function Definition ===");
    bridge.eval("(define (double x) (* x 2))")?;
    let result = bridge.eval("(double 21)")?;
    println!("(double 21) = {}", result);

    // Test lambda and higher-order functions
    println!("\n=== Lambda and Higher-Order Functions ===");
    bridge.eval("(define map-square (lambda (lst) (if (null? lst) '() (cons (call-external \"square\" (car lst)) (map-square (cdr lst))))))")?;
    let result = bridge.eval("(map-square '(1 2 3 4))")?;
    println!("(map-square '(1 2 3 4)) = {}", result);

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_external_functions() -> Result<()> {
        let mut bridge = LambdustBridge::new();

        bridge.register_function("add-one", Some(1), |args| {
            let n = match &args[0] {
                Value::Number(num) => match num {
                    lambdust::lexer::SchemeNumber::Integer(i) => *i,
                    _ => {
                        return Err(lambdust::error::LambdustError::TypeError(
                            "Expected integer".to_string(),
                        ));
                    }
                },
                _ => {
                    return Err(lambdust::error::LambdustError::TypeError(
                        "Expected number".to_string(),
                    ));
                }
            };
            Ok(Value::from(n + 1))
        });

        let result = bridge.eval("(call-external \"add-one\" 41)")?;
        assert_eq!(result, Value::from(42i64));

        Ok(())
    }

    #[test]
    fn test_object_registration() {
        let mut bridge = LambdustBridge::new();
        let calc = Calculator::new();
        let id = bridge.register_object(calc, "Calculator");
        assert!(id > 0);
    }
}
