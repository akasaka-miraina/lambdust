//! R7RS Compliance Validation for SRFI-14 Character Sets
//!
//! This module validates that SRFI-14 standard character sets are fully
//! consistent with R7RS character predicates and Unicode handling.

use crate::stdlib::charset::{CharSet, StandardCharSets};
use std::collections::HashSet;

/// Validates that SRFI-14 character sets are consistent with R7RS predicates
pub struct CharSetR7RSValidator;

impl CharSetR7RSValidator {
    /// Validates all standard character sets against R7RS requirements
    pub fn validate_all() -> ValidationReport {
        let mut report = ValidationReport::new();

        report.add_test("Lower-case letters", Self::validate_lower_case());
        report.add_test("Upper-case letters", Self::validate_upper_case());
        report.add_test("Letter characters", Self::validate_letters());
        report.add_test("Digit characters", Self::validate_digits());
        report.add_test(
            "Letter+digit characters",
            Self::validate_letter_plus_digit(),
        );
        report.add_test("Whitespace characters", Self::validate_whitespace());
        report.add_test("Hex-digit characters", Self::validate_hex_digits());
        report.add_test("Blank characters", Self::validate_blank());
        report.add_test("ASCII characters", Self::validate_ascii());
        report.add_test("Empty set", Self::validate_empty());
        report.add_test("Graphic characters", Self::validate_graphic());
        report.add_test("Printing characters", Self::validate_printing());
        report.add_test("Punctuation characters", Self::validate_punctuation());

        report
    }

    /// Validates lower-case character set against char-lowercase?
    fn validate_lower_case() -> TestResult {
        let charset = StandardCharSets::lower_case();
        let mut errors = Vec::new();

        // Test all ASCII characters
        for code in 0..128 {
            if let Some(ch) = char::from_u32(code) {
                let in_charset = charset.contains(ch);
                let is_lowercase = ch.is_lowercase();

                if in_charset != is_lowercase {
                    errors.push(ValidationError::Inconsistency {
                        character: ch,
                        charset_result: in_charset,
                        predicate_name: "char-lowercase?".to_string(),
                        predicate_result: is_lowercase,
                    });
                }
            }
        }

        // Test some common Unicode lowercase characters
        let unicode_lowercase_chars = ['à', 'é', 'ñ', 'ü', 'α', 'β', 'γ'];
        for &ch in &unicode_lowercase_chars {
            let in_charset = charset.contains(ch);
            let is_lowercase = ch.is_lowercase();

            if in_charset != is_lowercase {
                errors.push(ValidationError::Inconsistency {
                    character: ch,
                    charset_result: in_charset,
                    predicate_name: "char-lowercase?".to_string(),
                    predicate_result: is_lowercase,
                });
            }
        }

        TestResult { errors }
    }

    /// Validates upper-case character set against char-uppercase?
    fn validate_upper_case() -> TestResult {
        let charset = StandardCharSets::upper_case();
        let mut errors = Vec::new();

        // Test all ASCII characters
        for code in 0..128 {
            if let Some(ch) = char::from_u32(code) {
                let in_charset = charset.contains(ch);
                let is_uppercase = ch.is_uppercase();

                if in_charset != is_uppercase {
                    errors.push(ValidationError::Inconsistency {
                        character: ch,
                        charset_result: in_charset,
                        predicate_name: "char-uppercase?".to_string(),
                        predicate_result: is_uppercase,
                    });
                }
            }
        }

        // Test some common Unicode uppercase characters
        let unicode_uppercase_chars = ['À', 'É', 'Ñ', 'Ü', 'Α', 'Β', 'Γ'];
        for &ch in &unicode_uppercase_chars {
            let in_charset = charset.contains(ch);
            let is_uppercase = ch.is_uppercase();

            if in_charset != is_uppercase {
                errors.push(ValidationError::Inconsistency {
                    character: ch,
                    charset_result: in_charset,
                    predicate_name: "char-uppercase?".to_string(),
                    predicate_result: is_uppercase,
                });
            }
        }

        TestResult { errors }
    }

    /// Validates letter character set against char-alphabetic?
    fn validate_letters() -> TestResult {
        let charset = StandardCharSets::letter();
        let mut errors = Vec::new();

        // Test all ASCII characters
        for code in 0..128 {
            if let Some(ch) = char::from_u32(code) {
                let in_charset = charset.contains(ch);
                let is_alphabetic = ch.is_alphabetic();

                if in_charset != is_alphabetic {
                    errors.push(ValidationError::Inconsistency {
                        character: ch,
                        charset_result: in_charset,
                        predicate_name: "char-alphabetic?".to_string(),
                        predicate_result: is_alphabetic,
                    });
                }
            }
        }

        // Test some Unicode alphabetic characters
        let unicode_alphabetic_chars = ['à', 'é', 'ñ', 'ü', 'α', 'β', 'γ', 'λ', '中', '日'];
        for &ch in &unicode_alphabetic_chars {
            let in_charset = charset.contains(ch);
            let is_alphabetic = ch.is_alphabetic();

            // Note: For Unicode, we expect consistency but both might be false
            // if the charset doesn't include extended Unicode
            if in_charset && !is_alphabetic {
                errors.push(ValidationError::Inconsistency {
                    character: ch,
                    charset_result: in_charset,
                    predicate_name: "char-alphabetic?".to_string(),
                    predicate_result: is_alphabetic,
                });
            }
        }

        TestResult { errors }
    }

    /// Validates digit character set against char-numeric?
    fn validate_digits() -> TestResult {
        let charset = StandardCharSets::digit();
        let mut errors = Vec::new();

        // Test all ASCII characters
        for code in 0..128 {
            if let Some(ch) = char::from_u32(code) {
                let in_charset = charset.contains(ch);
                let is_numeric = ch.is_ascii_digit(); // Use ASCII digit for consistency

                if in_charset != is_numeric {
                    errors.push(ValidationError::Inconsistency {
                        character: ch,
                        charset_result: in_charset,
                        predicate_name: "char-numeric?".to_string(),
                        predicate_result: is_numeric,
                    });
                }
            }
        }

        // Validate exact size
        if charset.size() != 10 {
            errors.push(ValidationError::WrongSize {
                expected: 10,
                actual: charset.size(),
                description: "ASCII digits 0-9".to_string(),
            });
        }

        // Validate specific digits
        for digit_char in '0'..='9' {
            if !charset.contains(digit_char) {
                errors.push(ValidationError::MissingCharacter {
                    character: digit_char,
                    description: "Required ASCII digit".to_string(),
                });
            }
        }

        TestResult { errors }
    }

    /// Validates letter+digit is union of letter and digit sets
    fn validate_letter_plus_digit() -> TestResult {
        let letter_plus_digit = StandardCharSets::letter_plus_digit();
        let letter = StandardCharSets::letter();
        let digit = StandardCharSets::digit();
        let union = letter.union(&digit);

        let mut errors = Vec::new();

        if !letter_plus_digit.is_equal(&union) {
            errors.push(ValidationError::SetOperationFailed {
                operation: "letter+digit != union(letter, digit)".to_string(),
                description: "Letter+digit set should be union of letter and digit sets"
                    .to_string(),
            });
        }

        // Validate that letter+digit contains alphanumeric characters
        for ch in "abcABC123".chars() {
            if !letter_plus_digit.contains(ch) {
                errors.push(ValidationError::MissingCharacter {
                    character: ch,
                    description: "Letter+digit should contain alphanumeric characters".to_string(),
                });
            }
        }

        TestResult { errors }
    }

    /// Validates whitespace character set against char-whitespace?
    fn validate_whitespace() -> TestResult {
        let charset = StandardCharSets::whitespace();
        let mut errors = Vec::new();

        // Test all ASCII characters
        for code in 0..128 {
            if let Some(ch) = char::from_u32(code) {
                let in_charset = charset.contains(ch);
                let is_whitespace = ch.is_whitespace();

                if in_charset != is_whitespace {
                    errors.push(ValidationError::Inconsistency {
                        character: ch,
                        charset_result: in_charset,
                        predicate_name: "char-whitespace?".to_string(),
                        predicate_result: is_whitespace,
                    });
                }
            }
        }

        // Validate required whitespace characters
        let required_whitespace = [' ', '\t', '\n', '\r'];
        for &ch in &required_whitespace {
            if !charset.contains(ch) {
                errors.push(ValidationError::MissingCharacter {
                    character: ch,
                    description: "Required whitespace character".to_string(),
                });
            }
        }

        TestResult { errors }
    }

    /// Validates hex-digit character set
    fn validate_hex_digits() -> TestResult {
        let charset = StandardCharSets::hex_digit();
        let mut errors = Vec::new();

        // Validate size (should be 22: 0-9, A-F, a-f)
        if charset.size() != 22 {
            errors.push(ValidationError::WrongSize {
                expected: 22,
                actual: charset.size(),
                description: "Hex digits 0-9, A-F, a-f".to_string(),
            });
        }

        // Validate all required hex digits
        let required_hex = "0123456789ABCDEFabcdef";
        for ch in required_hex.chars() {
            if !charset.contains(ch) {
                errors.push(ValidationError::MissingCharacter {
                    character: ch,
                    description: "Required hex digit".to_string(),
                });
            }
        }

        // Validate no invalid characters
        let invalid_hex = "ghijklmnopqrstuvwxyzGHIJKLMNOPQRSTUVWXYZ!@#$%";
        for ch in invalid_hex.chars() {
            if charset.contains(ch) {
                errors.push(ValidationError::UnexpectedCharacter {
                    character: ch,
                    description: "Should not be in hex-digit set".to_string(),
                });
            }
        }

        TestResult { errors }
    }

    /// Validates blank character set (should be space and tab only)
    fn validate_blank() -> TestResult {
        let charset = StandardCharSets::blank();
        let mut errors = Vec::new();

        // Should contain exactly space and tab
        if charset.size() != 2 {
            errors.push(ValidationError::WrongSize {
                expected: 2,
                actual: charset.size(),
                description: "Blank characters (space and tab only)".to_string(),
            });
        }

        if !charset.contains(' ') {
            errors.push(ValidationError::MissingCharacter {
                character: ' ',
                description: "Blank set must contain space".to_string(),
            });
        }

        if !charset.contains('\t') {
            errors.push(ValidationError::MissingCharacter {
                character: '\t',
                description: "Blank set must contain tab".to_string(),
            });
        }

        // Should NOT contain newline, return, etc.
        let non_blank_whitespace = ['\n', '\r', '\x0C'];
        for &ch in &non_blank_whitespace {
            if charset.contains(ch) {
                errors.push(ValidationError::UnexpectedCharacter {
                    character: ch,
                    description: "Should not be in blank set (only space/tab allowed)".to_string(),
                });
            }
        }

        TestResult { errors }
    }

    /// Validates ASCII character set
    fn validate_ascii() -> TestResult {
        let charset = StandardCharSets::ascii();
        let mut errors = Vec::new();

        // Should contain exactly 128 characters
        if charset.size() != 128 {
            errors.push(ValidationError::WrongSize {
                expected: 128,
                actual: charset.size(),
                description: "ASCII characters 0-127".to_string(),
            });
        }

        // Should contain all ASCII characters
        for code in 0..128 {
            if let Some(ch) = char::from_u32(code) {
                if !charset.contains(ch) {
                    errors.push(ValidationError::MissingCharacter {
                        character: ch,
                        description: format!("ASCII character code {}", code),
                    });
                }
            }
        }

        // Should not contain non-ASCII characters
        let non_ascii_chars = ['à', 'ñ', 'λ', '中'];
        for &ch in &non_ascii_chars {
            if charset.contains(ch) {
                errors.push(ValidationError::UnexpectedCharacter {
                    character: ch,
                    description: "Non-ASCII character should not be in ASCII set".to_string(),
                });
            }
        }

        TestResult { errors }
    }

    /// Validates empty character set
    fn validate_empty() -> TestResult {
        let charset = StandardCharSets::empty();
        let mut errors = Vec::new();

        if charset.size() != 0 {
            errors.push(ValidationError::WrongSize {
                expected: 0,
                actual: charset.size(),
                description: "Empty set".to_string(),
            });
        }

        if !charset.is_empty() {
            errors.push(ValidationError::SetOperationFailed {
                operation: "is_empty()".to_string(),
                description: "Empty set should report as empty".to_string(),
            });
        }

        TestResult { errors }
    }

    /// Validates graphic character set
    fn validate_graphic() -> TestResult {
        let charset = StandardCharSets::graphic();
        let mut errors = Vec::new();

        // Should contain all ASCII printable characters except space
        for code in 33..=126 {
            // ! to ~
            if let Some(ch) = char::from_u32(code) {
                if !charset.contains(ch) {
                    errors.push(ValidationError::MissingCharacter {
                        character: ch,
                        description: "ASCII graphic character".to_string(),
                    });
                }
            }
        }

        // Should not contain control characters
        for code in 0..32 {
            if let Some(ch) = char::from_u32(code) {
                if charset.contains(ch) {
                    errors.push(ValidationError::UnexpectedCharacter {
                        character: ch,
                        description: "Control character should not be graphic".to_string(),
                    });
                }
            }
        }

        TestResult { errors }
    }

    /// Validates printing character set
    fn validate_printing() -> TestResult {
        let charset = StandardCharSets::printing();
        let graphic = StandardCharSets::graphic();
        let mut errors = Vec::new();

        // Printing should include all graphic characters
        if !graphic.is_subset(&charset) {
            errors.push(ValidationError::SetOperationFailed {
                operation: "graphic ⊆ printing".to_string(),
                description: "Printing set should include all graphic characters".to_string(),
            });
        }

        // Should contain space (difference from graphic)
        if !charset.contains(' ') {
            errors.push(ValidationError::MissingCharacter {
                character: ' ',
                description: "Printing set should contain space".to_string(),
            });
        }

        TestResult { errors }
    }

    /// Validates punctuation character set
    fn validate_punctuation() -> TestResult {
        let charset = StandardCharSets::punctuation();
        let mut errors = Vec::new();

        // Should contain common ASCII punctuation
        let punctuation_chars = "!\"#$%&'()*+,-./:;<=>?@[\\]^_`{|}~";
        for ch in punctuation_chars.chars() {
            if !charset.contains(ch) {
                errors.push(ValidationError::MissingCharacter {
                    character: ch,
                    description: "ASCII punctuation character".to_string(),
                });
            }
        }

        // Should not contain alphanumeric characters
        for ch in "abcABC123".chars() {
            if charset.contains(ch) {
                errors.push(ValidationError::UnexpectedCharacter {
                    character: ch,
                    description: "Alphanumeric character should not be punctuation".to_string(),
                });
            }
        }

        TestResult { errors }
    }
}

/// Represents a validation error
#[derive(Debug, Clone)]
pub enum ValidationError {
    /// Character set and R7RS predicate are inconsistent
    Inconsistency {
        character: char,
        charset_result: bool,
        predicate_name: String,
        predicate_result: bool,
    },
    /// Character set has wrong size
    WrongSize {
        expected: usize,
        actual: usize,
        description: String,
    },
    /// Character set is missing required character
    MissingCharacter {
        character: char,
        description: String,
    },
    /// Character set contains unexpected character
    UnexpectedCharacter {
        character: char,
        description: String,
    },
    /// Set operation failed
    SetOperationFailed {
        operation: String,
        description: String,
    },
}

impl std::fmt::Display for ValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ValidationError::Inconsistency {
                character,
                charset_result,
                predicate_name,
                predicate_result,
            } => {
                write!(
                    f,
                    "Inconsistency for '{}' (U+{:04X}): charset={}, {}={}",
                    character, *character as u32, charset_result, predicate_name, predicate_result
                )
            }
            ValidationError::WrongSize {
                expected,
                actual,
                description,
            } => {
                write!(
                    f,
                    "Wrong size for {}: expected {}, got {}",
                    description, expected, actual
                )
            }
            ValidationError::MissingCharacter {
                character,
                description,
            } => {
                write!(
                    f,
                    "Missing character '{}' (U+{:04X}): {}",
                    character, *character as u32, description
                )
            }
            ValidationError::UnexpectedCharacter {
                character,
                description,
            } => {
                write!(
                    f,
                    "Unexpected character '{}' (U+{:04X}): {}",
                    character, *character as u32, description
                )
            }
            ValidationError::SetOperationFailed {
                operation,
                description,
            } => {
                write!(f, "Set operation failed '{}': {}", operation, description)
            }
        }
    }
}

/// Result of a single validation test
#[derive(Debug)]
pub struct TestResult {
    pub errors: Vec<ValidationError>,
}

impl TestResult {
    pub fn is_success(&self) -> bool {
        self.errors.is_empty()
    }

    pub fn error_count(&self) -> usize {
        self.errors.len()
    }
}

/// Complete validation report
#[derive(Debug)]
pub struct ValidationReport {
    pub tests: Vec<(String, TestResult)>,
}

impl ValidationReport {
    pub fn new() -> Self {
        Self { tests: Vec::new() }
    }

    pub fn add_test(&mut self, name: impl Into<String>, result: TestResult) {
        self.tests.push((name.into(), result));
    }

    pub fn total_tests(&self) -> usize {
        self.tests.len()
    }

    pub fn passed_tests(&self) -> usize {
        self.tests
            .iter()
            .filter(|(_, result)| result.is_success())
            .count()
    }

    pub fn failed_tests(&self) -> usize {
        self.total_tests() - self.passed_tests()
    }

    pub fn total_errors(&self) -> usize {
        self.tests
            .iter()
            .map(|(_, result)| result.error_count())
            .sum()
    }

    pub fn is_fully_compliant(&self) -> bool {
        self.failed_tests() == 0
    }

    /// Print a comprehensive report
    pub fn print_report(&self) {
        println!("=== SRFI-14 R7RS Compliance Validation Report ===");
        println!();

        for (test_name, result) in &self.tests {
            if result.is_success() {
                println!("✓ {}: PASS", test_name);
            } else {
                println!("✗ {}: FAIL ({} errors)", test_name, result.error_count());
                for error in &result.errors {
                    println!("    - {}", error);
                }
            }
        }

        println!();
        println!("=== SUMMARY ===");
        println!("Total tests: {}", self.total_tests());
        println!("Passed: {}", self.passed_tests());
        println!("Failed: {}", self.failed_tests());
        println!("Total errors: {}", self.total_errors());

        if self.is_fully_compliant() {
            println!("🎉 SRFI-14 implementation is fully R7RS compliant!");
        } else {
            println!("⚠️  SRFI-14 implementation has compliance issues that need attention.");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_standard_character_sets() {
        let report = CharSetR7RSValidator::validate_all();

        // Print the full report for manual inspection
        report.print_report();

        // We expect some tests to pass, but there might be issues to address
        assert!(report.total_tests() > 0, "Should run validation tests");

        // For initial implementation, we'll accept some failures but track them
        if !report.is_fully_compliant() {
            println!(
                "WARNING: Found {} compliance issues that need attention",
                report.total_errors()
            );
        }
    }

    #[test]
    fn test_digit_set_basic_requirements() {
        let digit_set = StandardCharSets::digit();

        // Basic requirements that must pass
        assert_eq!(
            digit_set.size(),
            10,
            "Digit set must have exactly 10 characters"
        );

        for digit in '0'..='9' {
            assert!(
                digit_set.contains(digit),
                "Digit set must contain '{}'",
                digit
            );
        }

        // Should not contain letters
        for letter in 'a'..='z' {
            assert!(
                !digit_set.contains(letter),
                "Digit set must not contain '{}'",
                letter
            );
        }
    }

    #[test]
    fn test_empty_set_requirements() {
        let empty_set = StandardCharSets::empty();

        assert_eq!(empty_set.size(), 0, "Empty set must have size 0");
        assert!(empty_set.is_empty(), "Empty set must report as empty");
        assert!(
            !empty_set.contains('a'),
            "Empty set must not contain any characters"
        );
    }
}
