#![allow(missing_docs)]
//! Test data generators for property-based testing
//!
//! This module provides efficient generators for creating random Scheme values
//! used in property-based testing. The generators are designed to create
//! diverse test data while maintaining performance and memory efficiency.

use crate::ast::Literal;
use crate::eval::value::Value;
use crate::property_testing::Generator;
use rand::{Rng, SeedableRng};
use rand_xorshift::XorShiftRng;
use std::sync::Arc;

/// Main generator for Scheme values
pub struct SchemeValueGenerator {
    /// Controls the distribution of different value types
    type_weights: ValueTypeWeights,
}

/// Weights for different value types in generation
#[derive(Debug, Clone)]
pub struct ValueTypeWeights {
    pub number: u32,
    pub integer: u32,
    pub boolean: u32,
    pub character: u32,
    pub string: u32,
    pub symbol: u32,
    pub list: u32,
    pub vector: u32,
    pub nil: u32,
}

impl Default for ValueTypeWeights {
    fn default() -> Self {
        Self {
            number: 20,
            integer: 25,
            boolean: 10,
            character: 10,
            string: 15,
            symbol: 10,
            list: 20,
            vector: 15,
            nil: 5,
        }
    }
}

impl SchemeValueGenerator {
    /// Create a new generator with default weights
    pub fn new() -> Self {
        Self {
            type_weights: ValueTypeWeights::default(),
        }
    }

    /// Create a generator with custom type weights
    pub fn with_weights(weights: ValueTypeWeights) -> Self {
        Self {
            type_weights: weights,
        }
    }

    /// Generate a random number (float)
    fn generate_number(&self, rng: &mut XorShiftRng) -> Value {
        // Generate various types of numbers including edge cases
        match rng.gen_range(0..10) {
            0 => Value::number(0.0),
            1 => Value::number(-0.0),
            2 => Value::number(1.0),
            3 => Value::number(-1.0),
            4 => Value::number(f64::INFINITY),
            5 => Value::number(f64::NEG_INFINITY),
            6 => Value::number(f64::NAN),
            7 => Value::number(f64::MIN),
            8 => Value::number(f64::MAX),
            _ => Value::number(rng.r#gen::<f64>() * 1000.0 - 500.0),
        }
    }

    /// Generate a random integer
    fn generate_integer(&self, rng: &mut XorShiftRng) -> Value {
        match rng.gen_range(0..8) {
            0 => Value::integer(0),
            1 => Value::integer(1),
            2 => Value::integer(-1),
            3 => Value::integer(i64::MAX),
            4 => Value::integer(i64::MIN),
            5 => Value::integer(rng.gen_range(-1000..1000)),
            6 => Value::integer(rng.gen_range(i64::MIN / 2..i64::MAX / 2)),
            _ => Value::integer(rng.r#gen()),
        }
    }

    /// Generate a random boolean
    fn generate_boolean(&self, rng: &mut XorShiftRng) -> Value {
        Value::boolean(rng.r#gen())
    }

    /// Generate a random character
    fn generate_character(&self, rng: &mut XorShiftRng) -> Value {
        let chars = [
            ' ', '\t', '\n', '\r', // Whitespace
            'a', 'z', 'A', 'Z', // ASCII letters
            '0', '9', // ASCII digits
            '!', '@', '#', '$', '%', '^', '&', '*', // Special characters
            'λ', 'π', 'Ω',  // Unicode characters
            '\0', // Null character
        ];

        let ch = if rng.gen_bool(0.7) {
            chars[rng.gen_range(0..chars.len())]
        } else {
            // Generate random Unicode character
            char::from_u32(rng.gen_range(1..0x10000)).unwrap_or('?')
        };

        Value::Literal(Literal::Character(ch))
    }

    /// Generate a random string
    fn generate_string(&self, rng: &mut XorShiftRng, size: usize) -> Value {
        let length = rng.gen_range(0..=size.min(100));

        // Special cases
        if length == 0 || rng.gen_bool(0.1) {
            return Value::string("");
        }

        let mut string = String::with_capacity(length);
        for _ in 0..length {
            let ch = match rng.gen_range(0..10) {
                0..=5 => rng.gen_range(b'a'..=b'z') as char,
                6..=8 => rng.gen_range(b'A'..=b'Z') as char,
                9 => rng.gen_range(b'0'..=b'9') as char,
                _ => ' ',
            };
            string.push(ch);
        }

        Value::string(string)
    }

    /// Generate a random symbol
    fn generate_symbol(&self, rng: &mut XorShiftRng, size: usize) -> Value {
        let symbols = [
            "x", "y", "z", "foo", "bar", "baz", "test", "value", "result", "+", "-", "*", "/", "=",
            "<", ">", "<=", ">=", "car", "cdr", "cons", "list", "length", "append", "map",
            "filter", "fold", "reduce", "apply", "λ", "define", "let", "if", "cond", "case",
        ];

        let symbol_name = if rng.gen_bool(0.8) {
            symbols[rng.gen_range(0..symbols.len())].to_string()
        } else {
            // Generate random symbol name
            let length = rng.gen_range(1..=size.min(20));
            let mut name = String::with_capacity(length);
            for i in 0..length {
                let ch = if i == 0 {
                    // First character should be alphabetic
                    rng.gen_range(b'a'..=b'z') as char
                } else {
                    match rng.gen_range(0..3) {
                        0 => rng.gen_range(b'a'..=b'z') as char,
                        1 => rng.gen_range(b'0'..=b'9') as char,
                        _ => '-',
                    }
                };
                name.push(ch);
            }
            name
        };

        Value::symbol_from_str(symbol_name)
    }

    /// Generate a random list
    fn generate_list(&self, rng: &mut XorShiftRng, size: usize) -> Value {
        if size == 0 || rng.gen_bool(0.1) {
            return Value::Nil;
        }

        let length = rng.gen_range(0..=(size / 2).min(10));
        let mut elements = Vec::with_capacity(length);

        for _ in 0..length {
            elements.push(self.generate_recursive(rng, size / 2));
        }

        Value::list(elements)
    }

    /// Generate a random vector
    fn generate_vector(&self, rng: &mut XorShiftRng, size: usize) -> Value {
        let length = rng.gen_range(0..=(size / 2).min(10));
        let mut elements = Vec::with_capacity(length);

        for _ in 0..length {
            elements.push(self.generate_recursive(rng, size / 2));
        }

        Value::vector(elements)
    }

    /// Generate a value recursively (for compound structures)
    fn generate_recursive(&self, rng: &mut XorShiftRng, size: usize) -> Value {
        // Reduce recursion depth to avoid stack overflow
        if size == 0 {
            return self.generate_atomic(rng);
        }

        // Weighted random selection of value type
        let total_weight = self.type_weights.number
            + self.type_weights.integer
            + self.type_weights.boolean
            + self.type_weights.character
            + self.type_weights.string
            + self.type_weights.symbol
            + self.type_weights.list
            + self.type_weights.vector
            + self.type_weights.nil;

        let choice = rng.gen_range(0..total_weight);
        let mut current_weight = 0;

        current_weight += self.type_weights.number;
        if choice < current_weight {
            return self.generate_number(rng);
        }

        current_weight += self.type_weights.integer;
        if choice < current_weight {
            return self.generate_integer(rng);
        }

        current_weight += self.type_weights.boolean;
        if choice < current_weight {
            return self.generate_boolean(rng);
        }

        current_weight += self.type_weights.character;
        if choice < current_weight {
            return self.generate_character(rng);
        }

        current_weight += self.type_weights.string;
        if choice < current_weight {
            return self.generate_string(rng, size);
        }

        current_weight += self.type_weights.symbol;
        if choice < current_weight {
            return self.generate_symbol(rng, size);
        }

        current_weight += self.type_weights.list;
        if choice < current_weight {
            return self.generate_list(rng, size);
        }

        current_weight += self.type_weights.vector;
        if choice < current_weight {
            return self.generate_vector(rng, size);
        }

        // Default to nil
        Value::Nil
    }

    /// Generate only atomic (non-recursive) values
    fn generate_atomic(&self, rng: &mut XorShiftRng) -> Value {
        match rng.gen_range(0..7) {
            0 => self.generate_number(rng),
            1 => self.generate_integer(rng),
            2 => self.generate_boolean(rng),
            3 => self.generate_character(rng),
            4 => self.generate_string(rng, 5),
            5 => self.generate_symbol(rng, 5),
            _ => Value::Nil,
        }
    }
}

impl Default for SchemeValueGenerator {
    fn default() -> Self {
        Self::new()
    }
}

impl Generator<Value> for SchemeValueGenerator {
    fn generate(&self, rng: &mut XorShiftRng, size: usize) -> Value {
        self.generate_recursive(rng, size)
    }

    fn shrink(&self, value: &Value) -> Vec<Value> {
        let mut shrunk = Vec::new();

        if let Some(n) = value.as_number() {
            if n != 0.0 {
                shrunk.push(Value::number(0.0));
            }
            if n > 1.0 {
                shrunk.push(Value::number(n / 2.0));
                shrunk.push(Value::number(1.0));
            }
            if n < -1.0 {
                shrunk.push(Value::number(n / 2.0));
                shrunk.push(Value::number(-1.0));
            }
        } else if let Some(n) = value.as_integer() {
            if n != 0 {
                shrunk.push(Value::integer(0));
            }
            if n > 1 {
                shrunk.push(Value::integer(n / 2));
                shrunk.push(Value::integer(1));
            }
            if n < -1 {
                shrunk.push(Value::integer(n / 2));
                shrunk.push(Value::integer(-1));
            }
        } else if let Some(s) = value.as_string() {
            if s.is_empty() {
                return shrunk;
            }
            shrunk.push(Value::string(""));
            if s.len() > 1 {
                let mid = s.len() / 2;
                shrunk.push(Value::string(&s[..mid]));
                shrunk.push(Value::string(&s[mid..]));
            }
        } else if let Some(elements) = value.as_list() {
            if elements.is_empty() {
                return shrunk;
            }
            shrunk.push(Value::Nil);
            if elements.len() > 1 {
                let mid = elements.len() / 2;
                shrunk.push(Value::list(elements[..mid].to_vec()));
                shrunk.push(Value::list(elements[mid..].to_vec()));
            }
            // Also try shrinking each element
            for (i, elem) in elements.iter().enumerate() {
                let elem_shrunk = self.shrink(elem);
                for new_elem in elem_shrunk {
                    let mut new_elements = elements.clone();
                    new_elements[i] = new_elem;
                    shrunk.push(Value::list(new_elements));
                }
            }
        } else if value.is_vector() {
            // Skip vector shrinking for now - would need proper vector access method
            // shrunk.push(Value::vector(vec![]));
        }
        // No shrinking for other types

        shrunk
    }
}

/// Specialized generator for list values
pub struct ListGenerator {
    value_generator: SchemeValueGenerator,
    min_length: usize,
    max_length: usize,
}

impl ListGenerator {
    pub fn new(min_length: usize, max_length: usize) -> Self {
        Self {
            value_generator: SchemeValueGenerator::new(),
            min_length,
            max_length,
        }
    }
}

impl Generator<Value> for ListGenerator {
    fn generate(&self, rng: &mut XorShiftRng, size: usize) -> Value {
        let length = rng.gen_range(self.min_length..=self.max_length.min(size));
        let mut elements = Vec::with_capacity(length);

        for _ in 0..length {
            elements.push(self.value_generator.generate(rng, size / 2));
        }

        Value::list(elements)
    }

    fn shrink(&self, value: &Value) -> Vec<Value> {
        self.value_generator.shrink(value)
    }
}

/// Specialized generator for numeric values
pub struct NumberGenerator {
    min_value: f64,
    max_value: f64,
}

impl NumberGenerator {
    pub fn new(min_value: f64, max_value: f64) -> Self {
        Self {
            min_value,
            max_value,
        }
    }
}

impl Generator<Value> for NumberGenerator {
    fn generate(&self, rng: &mut XorShiftRng, _size: usize) -> Value {
        Value::number(rng.gen_range(self.min_value..=self.max_value))
    }

    fn shrink(&self, value: &Value) -> Vec<Value> {
        if let Some(n) = value.as_number() {
            // n is already extracted from as_number()
            let mut shrunk = Vec::new();

            if n != 0.0 && n >= self.min_value && 0.0 <= self.max_value {
                shrunk.push(Value::number(0.0));
            }

            if n > 1.0 {
                let half = n / 2.0;
                if half >= self.min_value {
                    shrunk.push(Value::number(half));
                }
            }

            shrunk
        } else {
            vec![]
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::SeedableRng;

    #[test]
    fn test_scheme_value_generator() {
        let generator = SchemeValueGenerator::new();
        let mut rng = XorShiftRng::seed_from_u64(42);

        for _ in 0..100 {
            let value = generator.generate(&mut rng, 10);
            // Just ensure we can generate values without panicking
            // Check that value is one of the expected types using is_* methods
            assert!(
                value.is_number()
                    || value.is_boolean()
                    || value.is_string()
                    || value.is_symbol()
                    || value.is_list()
                    || value.is_vector()
                    || value.is_nil()
                    || matches!(value, Value::Literal(_))
            );
        }
    }

    #[test]
    fn test_number_generator() {
        let generator = NumberGenerator::new(-10.0, 10.0);
        let mut rng = XorShiftRng::seed_from_u64(42);

        for _ in 0..50 {
            let value = generator.generate(&mut rng, 10);
            if let Some(n) = value.as_number() {
                assert!(n >= -10.0 && n <= 10.0);
            } else {
                panic!("NumberGenerator should only generate numbers");
            }
        }
    }

    #[test]
    fn test_list_generator() {
        let generator = ListGenerator::new(1, 5);
        let mut rng = XorShiftRng::seed_from_u64(42);

        for _ in 0..20 {
            let value = generator.generate(&mut rng, 10);
            if let Some(elements) = value.as_list() {
                assert!(elements.len() >= 1 && elements.len() <= 5);
            } else {
                panic!("ListGenerator should only generate lists");
            }
        }
    }

    #[test]
    fn test_shrinking() {
        let generator = SchemeValueGenerator::new();

        // Test number shrinking
        let big_number = Value::number(100.0);
        let shrunk = generator.shrink(&big_number);
        assert!(!shrunk.is_empty());
        assert!(shrunk.contains(&Value::number(0.0)));

        // Test list shrinking
        let big_list = Value::list(vec![
            Value::integer(1),
            Value::integer(2),
            Value::integer(3),
            Value::integer(4),
        ]);
        let shrunk = generator.shrink(&big_list);
        assert!(!shrunk.is_empty());
        assert!(shrunk.contains(&Value::Nil));
    }
}
