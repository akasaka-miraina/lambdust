//! Advanced shrinking algorithms for property-based testing
//!
//! This module provides sophisticated algorithms for minimizing failing test cases,
//! making it easier to identify the root cause of property violations.

use crate::eval::value::Value;
use std::collections::VecDeque;

/// Smart shrinking algorithm that uses delta debugging principles
pub struct SmartShrinker;

impl SmartShrinker {
    /// Shrink a value using advanced heuristics
    pub fn shrink_value(value: &Value) -> Vec<Value> {
        if value.is_number() {
            // Try to extract as f64
            if let Some(n) = value.as_integer() {
                Self::shrink_integer(n)
            } else {
                // Assume it's a floating point number - need to handle this case
                vec![Value::number(0.0)]
            }
        } else if let Some(s) = value.as_string() {
            Self::shrink_string(s)
        } else if value.is_list() {
            if let Some(elements) = value.as_list() {
                Self::shrink_list(&elements)
            } else {
                vec![]
            }
        } else if value.is_vector() {
            // Handle vector case - we need to add proper vector access method
            vec![]
        } else {
            vec![] // No shrinking for other types
        }
    }

    /// Shrink a floating-point number
    fn shrink_number(n: f64) -> Vec<Value> {
        let mut candidates = Vec::new();

        // Special values first
        if n != 0.0 && n.is_finite() {
            candidates.push(Value::number(0.0));
        }

        if n != 1.0 && n.is_finite() && n > 1.0 {
            candidates.push(Value::number(1.0));
        }

        if n != -1.0 && n.is_finite() && n < -1.0 {
            candidates.push(Value::number(-1.0));
        }

        // Geometric shrinking
        if n.abs() > 1.0 && n.is_finite() {
            candidates.push(Value::number(n / 2.0));
            candidates.push(Value::number(n * 0.9));
            candidates.push(Value::number(n * 0.1));
        }

        // Floor and ceiling for non-integers
        if n.fract() != 0.0 && n.is_finite() {
            candidates.push(Value::number(n.floor()));
            candidates.push(Value::number(n.ceil()));
        }

        // Towards zero
        if n > 0.0 && n.is_finite() {
            let smaller = n - 1.0;
            if smaller >= 0.0 {
                candidates.push(Value::number(smaller));
            }
        } else if n < 0.0 && n.is_finite() {
            let smaller = n + 1.0;
            if smaller <= 0.0 {
                candidates.push(Value::number(smaller));
            }
        }

        // Remove duplicates and sort by "simplicity" (absolute value)
        candidates.sort_by(|a, b| {
            if let (Some(a_val), Some(b_val)) = (a.as_number(), b.as_number()) {
                a_val.abs().partial_cmp(&b_val.abs()).unwrap_or(std::cmp::Ordering::Equal)
            } else {
                std::cmp::Ordering::Equal
            }
        });

        candidates.dedup();
        candidates
    }

    /// Shrink an integer
    fn shrink_integer(n: i64) -> Vec<Value> {
        let mut candidates = Vec::new();

        // Special values
        if n != 0 {
            candidates.push(Value::integer(0));
        }

        if n != 1 && n > 1 {
            candidates.push(Value::integer(1));
        }

        if n != -1 && n < -1 {
            candidates.push(Value::integer(-1));
        }

        // Binary shrinking (divide by 2)
        if n.abs() > 1 {
            candidates.push(Value::integer(n / 2));
        }

        // Linear shrinking (subtract/add 1)
        if n > 0 {
            candidates.push(Value::integer(n - 1));
        } else if n < 0 {
            candidates.push(Value::integer(n + 1));
        }

        // Powers of 2 shrinking
        if n.abs() > 2 {
            let log2 = (n.abs() as f64).log2() as i64;
            if log2 > 1 {
                let power_of_2 = 2i64.pow(log2 as u32 - 1);
                if n > 0 {
                    candidates.push(Value::integer(power_of_2));
                } else {
                    candidates.push(Value::integer(-power_of_2));
                }
            }
        }

        // Sort by absolute value (prefer smaller absolute values)
        candidates.sort_by_key(|v| {
            if let Some(val) = v.as_integer() {
                val.abs()
            } else {
                i64::MAX
            }
        });

        candidates.dedup();
        candidates
    }

    /// Shrink a string
    fn shrink_string(s: &str) -> Vec<Value> {
        let mut candidates = Vec::new();

        if s.is_empty() {
            return candidates;
        }

        // Empty string
        candidates.push(Value::string(""));

        // Remove characters from the end
        for i in (0..s.len()).rev() {
            if let Some(substr) = s.get(..i) {
                candidates.push(Value::string(substr));
            }
        }

        // Remove characters from the beginning
        for i in 1..s.len() {
            if let Some(substr) = s.get(i..) {
                candidates.push(Value::string(substr));
            }
        }

        // Binary shrinking - remove middle sections
        if s.len() > 2 {
            let mid = s.len() / 2;
            if let (Some(left), Some(right)) = (s.get(..mid), s.get(mid..)) {
                candidates.push(Value::string(left));
                candidates.push(Value::string(right));
            }

            // Try removing quarters
            let quarter = s.len() / 4;
            if quarter > 0 {
                for start in &[0, quarter, 2 * quarter, 3 * quarter] {
                    let end = (start + quarter).min(s.len());
                    if let (Some(before), Some(after)) = (s.get(..*start), s.get(end..)) {
                        candidates.push(Value::string(&format!("{}{}", before, after)));
                    }
                }
            }
        }

        // Simplify characters (replace with simpler characters)
        for (i, ch) in s.char_indices() {
            let simpler_chars = Self::get_simpler_chars(ch);
            for simple_ch in simpler_chars {
                let mut new_string = String::with_capacity(s.len());
                new_string.push_str(&s[..i]);
                new_string.push(simple_ch);
                if let Some(rest) = s.get(i + ch.len_utf8()..) {
                    new_string.push_str(rest);
                }
                candidates.push(Value::string(new_string));
            }
        }

        // Sort by length (prefer shorter strings)
        candidates.sort_by_key(|v| {
            if let Some(val) = v.as_string() {
                val.len()
            } else {
                usize::MAX
            }
        });

        candidates.dedup();
        candidates
    }

    /// Get simpler characters for character simplification
    fn get_simpler_chars(ch: char) -> Vec<char> {
        match ch {
            'A'..='Z' => vec!['A', 'a'],
            'a'..='z' => vec!['a'],
            '0'..='9' => vec!['0', '1'],
            ' ' => vec![],
            _ => vec![' ', 'a', '0'],
        }
    }

    /// Shrink a list using advanced techniques
    fn shrink_list(elements: &[Value]) -> Vec<Value> {
        let mut candidates = Vec::new();

        if elements.is_empty() {
            return candidates;
        }

        // Empty list
        candidates.push(Value::Nil);

        // Single elements
        for element in elements {
            candidates.push(element.clone());
        }

        // Remove elements from different positions
        for i in 0..elements.len() {
            let mut new_elements = elements.to_vec();
            new_elements.remove(i);
            candidates.push(Value::list(new_elements));
        }

        // Binary shrinking - split in half
        if elements.len() > 1 {
            let mid = elements.len() / 2;
            candidates.push(Value::list(elements[..mid].to_vec()));
            candidates.push(Value::list(elements[mid..].to_vec()));
        }

        // Remove multiple consecutive elements
        if elements.len() > 2 {
            for start in 0..elements.len() {
                for length in 1..=(elements.len() - start) {
                    if start == 0 && length == elements.len() {
                        continue; // Skip removing all elements (already handled)
                    }

                    let mut new_elements = Vec::new();
                    new_elements.extend_from_slice(&elements[..start]);
                    new_elements.extend_from_slice(&elements[start + length..]);
                    candidates.push(Value::list(new_elements));
                }
            }
        }

        // Delta debugging - try to find minimal failing subset
        candidates.extend(Self::delta_debug_list(elements));

        // Sort by length (prefer shorter lists)
        candidates.sort_by_key(|v| {
            if let Some(val) = v.as_list() {
                val.len()
            } else if v.is_nil() {
                0
            } else {
                usize::MAX
            }
        });

        candidates.dedup();
        candidates
    }

    /// Apply delta debugging to find minimal failing sublists
    fn delta_debug_list(elements: &[Value]) -> Vec<Value> {
        let mut candidates = Vec::new();

        // Use a queue-based approach for systematic exploration
        let mut queue = VecDeque::new();
        queue.push_back(elements.to_vec());

        while let Some(current) = queue.pop_front() {
            if current.len() <= 1 {
                continue;
            }

            // Try splitting into complementary subsets
            let n = current.len();
            let subset_size = (n / 2).max(1);

            // Generate all subsets of the given size
            for start in 0..=(n - subset_size) {
                let subset = current[start..start + subset_size].to_vec();
                candidates.push(Value::list(subset.clone()));

                // Add subset to queue for further reduction
                if subset.len() > 1 {
                    queue.push_back(subset);
                }
            }
        }

        candidates
    }

    /// Shrink a vector (similar to list but produces vectors)
    fn shrink_vector(elements: &[Value]) -> Vec<Value> {
        let list_candidates = Self::shrink_list(elements);
        
        list_candidates
            .into_iter()
            .map(|candidate| {
                if let Some(elements) = candidate.as_list() {
                    Value::vector(elements)
                } else if candidate.is_nil() {
                    Value::vector(vec![])
                } else {
                    candidate // Single elements remain as they are
                }
            })
            .collect()
    }
}

/// Shrinking strategy that focuses on structural simplification
pub struct StructuralShrinker;

impl StructuralShrinker {
    /// Shrink focusing on structural complexity
    pub fn shrink_structurally(value: &Value) -> Vec<Value> {
        if let Some(elements) = value.as_list() {
            Self::shrink_list_structure(&elements)
        } else if value.is_vector() {
            // For now, skip vector shrinking - would need proper vector access method
            SmartShrinker::shrink_value(value)
        } else {
            SmartShrinker::shrink_value(value)
        }
    }

    /// Shrink list by focusing on structural patterns
    fn shrink_list_structure(elements: &[Value]) -> Vec<Value> {
        let mut candidates = Vec::new();

        if elements.is_empty() {
            return candidates;
        }

        // Structural patterns
        candidates.push(Value::Nil);

        // Keep only the first element
        candidates.push(elements[0].clone());

        // Keep only the last element
        if elements.len() > 1 {
            candidates.push(elements[elements.len() - 1].clone());
        }

        // Create flat structure from nested structures
        let mut flattened = Vec::new();
        for element in elements {
            if let Some(sub_elements) = element.as_list() {
                flattened.extend(sub_elements);
            } else {
                flattened.push(element.clone());
            }
        }

        if flattened.len() != elements.len() {
            candidates.push(Value::list(flattened));
        }

        // Remove nested structures
        let simplified: Vec<Value> = elements
            .iter()
            .filter_map(|element| match element {
                v if v.is_list() || v.is_vector() => None,
                other => Some(other.clone()),
            })
            .collect();

        if !simplified.is_empty() && simplified.len() < elements.len() {
            candidates.push(Value::list(simplified));
        }

        candidates
    }

    /// Shrink vector by focusing on structural patterns
    fn shrink_vector_structure(elements: &[Value]) -> Vec<Value> {
        let list_candidates = Self::shrink_list_structure(elements);
        
        list_candidates
            .into_iter()
            .map(|candidate| {
                if let Some(elements) = candidate.as_list() {
                    Value::vector(elements)
                } else if candidate.is_nil() {
                    Value::vector(vec![])
                } else {
                    candidate
                }
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_number_shrinking() {
        let candidates = SmartShrinker::shrink_number(100.0);
        assert!(candidates.contains(&Value::number(0.0)));
        assert!(candidates.contains(&Value::number(1.0)));
        assert!(candidates.contains(&Value::number(50.0)));
    }

    #[test]
    fn test_integer_shrinking() {
        let candidates = SmartShrinker::shrink_integer(1000);
        assert!(candidates.contains(&Value::integer(0)));
        assert!(candidates.contains(&Value::integer(1)));
        assert!(candidates.contains(&Value::integer(500)));
    }

    #[test]
    fn test_string_shrinking() {
        let candidates = SmartShrinker::shrink_string("hello world");
        assert!(candidates.contains(&Value::string("")));
        assert!(candidates.contains(&Value::string("hello")));
        assert!(candidates.contains(&Value::string("world")));
    }

    #[test]
    fn test_list_shrinking() {
        let original = vec![
            Value::integer(1),
            Value::integer(2),
            Value::integer(3),
            Value::integer(4),
        ];
        let candidates = SmartShrinker::shrink_list(&original);
        
        assert!(candidates.contains(&Value::Nil));
        assert!(candidates.contains(&Value::integer(1)));
        assert!(candidates.contains(&Value::list(vec![Value::integer(1), Value::integer(2)])));
    }

    #[test]
    fn test_structural_shrinking() {
        let nested_list = Value::list(vec![
            Value::list(vec![Value::integer(1), Value::integer(2)]),
            Value::list(vec![Value::integer(3), Value::integer(4)]),
        ]);

        if let Some(elements) = nested_list.as_list() {
            let candidates = StructuralShrinker::shrink_list_structure(&elements);
            
            // Should contain flattened version
            let expected_flat = Value::list(vec![
                Value::integer(1),
                Value::integer(2),
                Value::integer(3),
                Value::integer(4),
            ]);
            
            assert!(candidates.contains(&expected_flat));
        }
    }

    #[test]
    fn test_delta_debugging() {
        let elements = vec![
            Value::integer(1),
            Value::integer(2),
            Value::integer(3),
            Value::integer(4),
            Value::integer(5),
            Value::integer(6),
        ];
        
        let candidates = SmartShrinker::delta_debug_list(&elements);
        
        // Should generate various subset candidates
        assert!(!candidates.is_empty());
        
        // Check that we have some expected subsets
        let expected_half = Value::list(vec![
            Value::integer(1),
            Value::integer(2),
            Value::integer(3),
        ]);
        
        assert!(candidates.contains(&expected_half));
    }
}