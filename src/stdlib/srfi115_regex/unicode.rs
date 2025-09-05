//! Unicode Support for SRFI-115
//!
//! Basic Unicode property matching implementation for SRFI-115.

use crate::stdlib::srfi115_regex::{
    error::{RegexError, RegexResult},
};

/// Unicode Property Type
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum UnicodeProperty {
    GeneralCategory(String),
    Script(String),
    Block(String),
    Binary(String),
}

impl UnicodeProperty {
    pub fn from_str(property_type: &str, value: &str) -> RegexResult<Self> {
        match property_type.to_lowercase().as_str() {
            "gc" | "general_category" => Ok(Self::GeneralCategory(value.to_string())),
            "sc" | "script" => Ok(Self::Script(value.to_string())),
            "blk" | "block" => Ok(Self::Block(value.to_string())),
            _ => Ok(Self::Binary(format!("{}={}", property_type, value))),
        }
    }

    pub fn from_single_str(s: &str) -> RegexResult<Self> {
        match s {
            "Lu" => Ok(Self::GeneralCategory("Lu".to_string())),
            "Ll" => Ok(Self::GeneralCategory("Ll".to_string())),
            "Latin" => Ok(Self::Script("Latin".to_string())),
            "ASCII" => Ok(Self::Binary("ASCII".to_string())),
            _ => Err(RegexError::invalid_unicode(s, "unknown Unicode property")),
        }
    }
}

pub struct UnicodePropertyMatcher;

impl UnicodePropertyMatcher {
    pub fn new() -> Self {
        Self
    }

    pub fn matches(&self, ch: char, property: &UnicodeProperty) -> bool {
        match property {
            UnicodeProperty::GeneralCategory(gc) => match gc.as_str() {
                "Lu" => ch.is_uppercase(),
                "Ll" => ch.is_lowercase(),
                _ => false,
            },
            UnicodeProperty::Script(script) => match script.as_str() {
                "Latin" => ch.is_ascii_alphabetic(),
                _ => false,
            },
            UnicodeProperty::Binary(prop) => match prop.as_str() {
                "ASCII" => ch.is_ascii(),
                _ => false,
            },
            _ => false,
        }
    }
}

impl Default for UnicodePropertyMatcher {
    fn default() -> Self {
        Self::new()
    }
}