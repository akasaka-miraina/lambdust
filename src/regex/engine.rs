//! Thompson NFA engine for regular expression matching.
//!
//! This module implements Thompson's construction algorithm to convert regular
//! expression patterns into Nondeterministic Finite Automata (NFA). The
//! Thompson NFA provides:
//!
//! - **O(m) Construction**: Linear in pattern length
//! - **O(nm) Matching**: Linear in text and pattern length  
//! - **Predictable Performance**: No exponential worst-case behavior
//! - **Simple Implementation**: Easy to understand and maintain
//!
//! ## Algorithm Overview
//!
//! Thompson's construction builds an NFA by recursively combining smaller
//! automata for each regex component:
//!
//! 1. **Base Cases**: Single characters, epsilon transitions
//! 2. **Concatenation**: Connect automata in sequence
//! 3. **Alternation**: Create parallel paths with epsilon transitions
//! 4. **Repetition**: Add loops with epsilon transitions
//!
//! The resulting NFA has exactly one start state and one accept state,
//! making composition straightforward.

use std::collections::{HashMap, HashSet};
use std::fmt;
use crate::regex::parser::{Pattern, PatternNode};

/// Error type for NFA engine operations.
#[derive(Debug, Clone)]
pub enum EngineError {
    /// Pattern too complex (e.g., too many states)
    TooComplex(String),
    /// Unsupported regex feature
    UnsupportedFeature(String),
    /// Internal engine error
    InternalError(String),
}

impl fmt::Display for EngineError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EngineError::TooComplex(msg) => write!(f, "Pattern too complex: {msg}"),
            EngineError::UnsupportedFeature(msg) => write!(f, "Unsupported feature: {msg}"),
            EngineError::InternalError(msg) => write!(f, "Internal error: {msg}"),
        }
    }
}

impl std::error::Error for EngineError {}

/// NFA state identifier.
pub type StateId = usize;

/// NFA transition types.
#[derive(Debug, Clone, PartialEq)]
pub enum Transition {
    /// Epsilon transition (no input consumed)
    Epsilon,
    /// Character match transition
    Char(char),
    /// Character class transition  
    CharClass(CharClass),
    /// Any character (except newline by default)
    Any,
    /// Start of string/line anchor
    Start,
    /// End of string/line anchor
    End,
}

/// Character class specification for efficient matching.
#[derive(Debug, Clone, PartialEq)]
pub struct CharClass {
    /// Set of individual characters
    chars: HashSet<char>,
    /// Character ranges (start, end) inclusive
    ranges: Vec<(char, char)>,
    /// Whether this is a negated class
    negated: bool,
    /// Built-in character classes
    builtin: Vec<BuiltinClass>,
}

/// Built-in character classes for common patterns.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BuiltinClass {
    /// \d - ASCII digits [0-9]
    Digit,
    /// \w - Word characters [a-zA-Z0-9_]
    Word, 
    /// \s - Whitespace characters [ \t\n\r\f]
    Space,
}

/// NFA state representation.
#[derive(Debug, Clone)]
pub struct NfaState {
    /// State identifier
    pub id: StateId,
    /// Outgoing transitions
    pub transitions: Vec<(Transition, StateId)>,
    /// Whether this is an accept state
    pub is_accept: bool,
}

/// Complete NFA representation.
#[derive(Debug, Clone)]
pub struct Nfa {
    /// All states in the NFA
    pub states: HashMap<StateId, NfaState>,
    /// Start state identifier
    pub start_state: StateId,
    /// Accept state identifiers
    pub accept_states: HashSet<StateId>,
    /// Next available state ID for construction
    next_state_id: StateId,
}

/// NFA engine for pattern compilation and execution coordination.
#[derive(Debug, Clone)]
pub struct NfaEngine {
    /// Compiled NFA
    nfa: Nfa,
    /// Original pattern for debugging
    pattern_string: String,
}

impl Default for CharClass {
    fn default() -> Self {
        Self::new()
    }
}

impl CharClass {
    /// Creates a new empty character class.
    pub fn new() -> Self {
        Self {
            chars: HashSet::new(),
            ranges: Vec::new(),
            negated: false,
            builtin: Vec::new(),
        }
    }
    
    /// Creates a character class from a single character.
    pub fn single(ch: char) -> Self {
        let mut chars = HashSet::new();
        chars.insert(ch);
        Self {
            chars,
            ranges: Vec::new(),
            negated: false,
            builtin: Vec::new(),
        }
    }
    
    /// Creates a character class from a range.
    pub fn range(start: char, end: char) -> Self {
        Self {
            chars: HashSet::new(),
            ranges: vec![(start, end)],
            negated: false,
            builtin: Vec::new(),
        }
    }
    
    /// Creates a built-in character class.
    pub fn builtin(class: BuiltinClass) -> Self {
        Self {
            chars: HashSet::new(),
            ranges: Vec::new(),
            negated: false,
            builtin: vec![class],
        }
    }
    
    /// Negates this character class.
    pub fn negate(mut self) -> Self {
        self.negated = !self.negated;
        self
    }
    
    /// Adds a character to this class.
    pub fn add_char(&mut self, ch: char) {
        self.chars.insert(ch);
    }
    
    /// Adds a character range to this class.
    pub fn add_range(&mut self, start: char, end: char) {
        self.ranges.push((start, end));
    }
    
    /// Tests whether a character matches this class.
    pub fn matches(&self, ch: char) -> bool {
        let mut matched = false;
        
        // Check individual characters
        if self.chars.contains(&ch) {
            matched = true;
        }
        
        // Check character ranges
        for &(start, end) in &self.ranges {
            if ch >= start && ch <= end {
                matched = true;
                break;
            }
        }
        
        // Check built-in classes
        for &builtin in &self.builtin {
            if builtin.matches(ch) {
                matched = true;
                break;
            }
        }
        
        // Apply negation
        if self.negated {
            !matched
        } else {
            matched
        }
    }
}

impl BuiltinClass {
    /// Tests whether a character matches this built-in class.
    pub fn matches(&self, ch: char) -> bool {
        match self {
            BuiltinClass::Digit => ch.is_ascii_digit(),
            BuiltinClass::Word => ch.is_ascii_alphanumeric() || ch == '_',
            BuiltinClass::Space => matches!(ch, ' ' | '\t' | '\n' | '\r' | '\x0C'),
        }
    }
}

impl NfaState {
    /// Creates a new NFA state.
    pub fn new(id: StateId) -> Self {
        Self {
            id,
            transitions: Vec::new(),
            is_accept: false,
        }
    }
    
    /// Adds a transition to this state.
    pub fn add_transition(&mut self, transition: Transition, target: StateId) {
        self.transitions.push((transition, target));
    }
    
    /// Marks this state as an accept state.
    pub fn set_accept(&mut self, accept: bool) {
        self.is_accept = accept;
    }
}

impl Default for Nfa {
    fn default() -> Self {
        Self::new()
    }
}

impl Nfa {
    /// Creates a new empty NFA.
    pub fn new() -> Self {
        Self {
            states: HashMap::new(),
            start_state: 0,
            accept_states: HashSet::new(),
            next_state_id: 0,
        }
    }
    
    /// Allocates a new state ID.
    fn alloc_state(&mut self) -> StateId {
        let id = self.next_state_id;
        self.next_state_id += 1;
        id
    }
    
    /// Adds a state to the NFA.
    pub fn add_state(&mut self, mut state: NfaState) -> StateId {
        let id = state.id;
        if state.is_accept {
            self.accept_states.insert(id);
        }
        self.states.insert(id, state);
        id
    }
    
    /// Creates a state and adds it to the NFA.
    pub fn create_state(&mut self) -> StateId {
        let id = self.alloc_state();
        let state = NfaState::new(id);
        self.add_state(state);
        id
    }
    
    /// Gets a mutable reference to a state.
    pub fn get_state_mut(&mut self, id: StateId) -> Option<&mut NfaState> {
        self.states.get_mut(&id)
    }
    
    /// Sets the start state.
    pub fn set_start_state(&mut self, id: StateId) {
        self.start_state = id;
    }
    
    /// Marks a state as accepting.
    pub fn set_accept_state(&mut self, id: StateId) {
        if let Some(state) = self.states.get_mut(&id) {
            state.set_accept(true);
            self.accept_states.insert(id);
        }
    }
    
    /// Thompson construction: creates NFA for a single character.
    pub fn from_char(ch: char) -> Self {
        let mut nfa = Self::new();
        
        let start = nfa.create_state();
        let accept = nfa.create_state();
        
        nfa.set_start_state(start);
        nfa.set_accept_state(accept);
        
        // Add character transition
        nfa.get_state_mut(start).unwrap()
            .add_transition(Transition::Char(ch), accept);
        
        nfa
    }
    
    /// Thompson construction: creates NFA for a character class.
    pub fn from_char_class(class: CharClass) -> Self {
        let mut nfa = Self::new();
        
        let start = nfa.create_state();
        let accept = nfa.create_state();
        
        nfa.set_start_state(start);
        nfa.set_accept_state(accept);
        
        // Add character class transition
        nfa.get_state_mut(start).unwrap()
            .add_transition(Transition::CharClass(class), accept);
        
        nfa
    }
    
    /// Thompson construction: creates NFA for concatenation.
    pub fn concat(mut first: Self, mut second: Self) -> Self {
        // Connect first's accept states to second's start state via epsilon
        for &accept_id in first.accept_states.clone().iter() {
            first.get_state_mut(accept_id).unwrap().set_accept(false);
            first.get_state_mut(accept_id).unwrap()
                .add_transition(Transition::Epsilon, second.start_state);
        }
        
        // Merge states, adjusting IDs to avoid conflicts
        let id_offset = first.next_state_id;
        
        for (old_id, mut state) in second.states {
            let new_id = old_id + id_offset;
            state.id = new_id;
            
            // Adjust transition targets
            for (_, target) in &mut state.transitions {
                *target += id_offset;
            }
            
            first.states.insert(new_id, state);
        }
        
        // Update accept states
        first.accept_states.clear();
        for &old_accept in &second.accept_states {
            first.accept_states.insert(old_accept + id_offset);
        }
        
        first.next_state_id += second.next_state_id;
        first
    }
    
    /// Thompson construction: creates NFA for alternation (|).
    pub fn alternate(mut first: Self, mut second: Self) -> Result<Self, EngineError> {
        let mut result = Self::new();
        
        let new_start = result.create_state();
        let new_accept = result.create_state();
        
        result.set_start_state(new_start);
        result.set_accept_state(new_accept);
        
        // Merge first NFA states with ID offset
        let first_offset = result.next_state_id;
        for (old_id, mut state) in first.states {
            let new_id = old_id + first_offset;
            state.id = new_id;
            
            // Adjust transition targets
            for (_, target) in &mut state.transitions {
                *target += first_offset;
            }
            
            result.states.insert(new_id, state);
        }
        result.next_state_id += first.next_state_id;
        
        // Merge second NFA states with ID offset
        let second_offset = result.next_state_id;
        for (old_id, mut state) in second.states {
            let new_id = old_id + second_offset;
            state.id = new_id;
            
            // Adjust transition targets  
            for (_, target) in &mut state.transitions {
                *target += second_offset;
            }
            
            result.states.insert(new_id, state);
        }
        result.next_state_id += second.next_state_id;
        
        // Connect new start to both sub-NFAs
        result.get_state_mut(new_start).unwrap()
            .add_transition(Transition::Epsilon, first.start_state + first_offset);
        result.get_state_mut(new_start).unwrap()
            .add_transition(Transition::Epsilon, second.start_state + second_offset);
        
        // Connect both sub-NFA accept states to new accept
        for &accept_id in &first.accept_states {
            let new_accept_id = accept_id + first_offset;
            result.get_state_mut(new_accept_id).unwrap().set_accept(false);
            result.get_state_mut(new_accept_id).unwrap()
                .add_transition(Transition::Epsilon, new_accept);
        }
        
        for &accept_id in &second.accept_states {
            let new_accept_id = accept_id + second_offset;
            result.get_state_mut(new_accept_id).unwrap().set_accept(false);
            result.get_state_mut(new_accept_id).unwrap()
                .add_transition(Transition::Epsilon, new_accept);
        }
        
        Ok(result)
    }
    
    /// Thompson construction: creates NFA for Kleene star (*).
    pub fn kleene_star(mut inner: Self) -> Self {
        let mut result = Self::new();
        
        let new_start = result.create_state();
        let new_accept = result.create_state();
        
        result.set_start_state(new_start);
        result.set_accept_state(new_accept);
        
        // Merge inner NFA states
        let offset = result.next_state_id;
        for (old_id, mut state) in inner.states {
            let new_id = old_id + offset;
            state.id = new_id;
            
            // Adjust transition targets
            for (_, target) in &mut state.transitions {
                *target += offset;
            }
            
            result.states.insert(new_id, state);
        }
        result.next_state_id += inner.next_state_id;
        
        // Connect new start to inner start and new accept (0 matches)
        result.get_state_mut(new_start).unwrap()
            .add_transition(Transition::Epsilon, inner.start_state + offset);
        result.get_state_mut(new_start).unwrap()
            .add_transition(Transition::Epsilon, new_accept);
        
        // Connect inner accept states to inner start (loop) and new accept
        for &accept_id in &inner.accept_states {
            let new_accept_id = accept_id + offset;
            result.get_state_mut(new_accept_id).unwrap().set_accept(false);
            result.get_state_mut(new_accept_id).unwrap()
                .add_transition(Transition::Epsilon, inner.start_state + offset);
            result.get_state_mut(new_accept_id).unwrap()
                .add_transition(Transition::Epsilon, new_accept);
        }
        
        result
    }
    
    /// Thompson construction: creates NFA for one-or-more (+).
    pub fn one_or_more(inner: Self) -> Self {
        // A+ is equivalent to AA*
        let star_part = Self::kleene_star(inner.clone());
        Self::concat(inner, star_part)
    }
    
    /// Thompson construction: creates NFA for zero-or-one (?).
    pub fn zero_or_one(mut inner: Self) -> Self {
        let mut result = Self::new();
        
        let new_start = result.create_state();
        let new_accept = result.create_state();
        
        result.set_start_state(new_start);
        result.set_accept_state(new_accept);
        
        // Merge inner NFA states
        let offset = result.next_state_id;
        for (old_id, mut state) in inner.states {
            let new_id = old_id + offset;
            state.id = new_id;
            
            // Adjust transition targets
            for (_, target) in &mut state.transitions {
                *target += offset;
            }
            
            result.states.insert(new_id, state);
        }
        result.next_state_id += inner.next_state_id;
        
        // Connect new start to inner start and new accept (0 matches)
        result.get_state_mut(new_start).unwrap()
            .add_transition(Transition::Epsilon, inner.start_state + offset);
        result.get_state_mut(new_start).unwrap()
            .add_transition(Transition::Epsilon, new_accept);
        
        // Connect inner accept states to new accept
        for &accept_id in &inner.accept_states {
            let new_accept_id = accept_id + offset;
            result.get_state_mut(new_accept_id).unwrap().set_accept(false);
            result.get_state_mut(new_accept_id).unwrap()
                .add_transition(Transition::Epsilon, new_accept);
        }
        
        result
    }
}

impl NfaEngine {
    /// Creates a new NFA engine from a parsed pattern.
    pub fn from_pattern(pattern: &Pattern) -> Result<Self, EngineError> {
        let nfa = Self::build_nfa(&pattern.root)?;
        
        Ok(Self {
            nfa,
            pattern_string: pattern.source.clone(),
        })
    }
    
    /// Recursively builds NFA from pattern AST.
    fn build_nfa(node: &PatternNode) -> Result<Nfa, EngineError> {
        match node {
            PatternNode::Char(ch) => Ok(Nfa::from_char(*ch)),
            PatternNode::CharClass(class) => Ok(Nfa::from_char_class(class.clone())),
            PatternNode::Any => Ok(Nfa::from_char_class(
                CharClass::new().negate() // Match anything except nothing
            )),
            
            PatternNode::Concat(parts) => {
                if parts.is_empty() {
                    return Err(EngineError::InternalError("Empty concatenation".to_string()));
                }
                
                let mut result = Self::build_nfa(&parts[0])?;
                for part in parts.iter().skip(1) {
                    let part_nfa = Self::build_nfa(part)?;
                    result = Nfa::concat(result, part_nfa);
                }
                Ok(result)
            }
            
            PatternNode::Alternate(alternatives) => {
                if alternatives.is_empty() {
                    return Err(EngineError::InternalError("Empty alternation".to_string()));
                }
                
                let mut result = Self::build_nfa(&alternatives[0])?;
                for alt in alternatives.iter().skip(1) {
                    let alt_nfa = Self::build_nfa(alt)?;
                    result = Nfa::alternate(result, alt_nfa)?;
                }
                Ok(result)
            }
            
            PatternNode::Star(inner) => {
                let inner_nfa = Self::build_nfa(inner)?;
                Ok(Nfa::kleene_star(inner_nfa))
            }
            
            PatternNode::Plus(inner) => {
                let inner_nfa = Self::build_nfa(inner)?;
                Ok(Nfa::one_or_more(inner_nfa))
            }
            
            PatternNode::Question(inner) => {
                let inner_nfa = Self::build_nfa(inner)?;
                Ok(Nfa::zero_or_one(inner_nfa))
            }
            
            _ => Err(EngineError::UnsupportedFeature(
                format!("Pattern node {node:?} not yet implemented")
            )),
        }
    }
    
    /// Gets the compiled NFA.
    pub fn nfa(&self) -> &Nfa {
        &self.nfa
    }
    
    /// Gets the original pattern string.
    pub fn pattern_string(&self) -> &str {
        &self.pattern_string
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_char_class_matching() {
        let mut class = CharClass::new();
        class.add_char('a');
        class.add_range('0', '9');
        
        assert!(class.matches('a'));
        assert!(class.matches('5'));
        assert!(!class.matches('b'));
        assert!(!class.matches('z'));
    }
    
    #[test]
    fn test_builtin_classes() {
        assert!(BuiltinClass::Digit.matches('5'));
        assert!(!BuiltinClass::Digit.matches('a'));
        
        assert!(BuiltinClass::Word.matches('a'));
        assert!(BuiltinClass::Word.matches('_'));
        assert!(!BuiltinClass::Word.matches('@'));
        
        assert!(BuiltinClass::Space.matches(' '));
        assert!(BuiltinClass::Space.matches('\t'));
        assert!(!BuiltinClass::Space.matches('a'));
    }
    
    #[test]
    fn test_single_char_nfa() {
        let nfa = Nfa::from_char('a');
        assert_eq!(nfa.states.len(), 2); // start + accept
        assert_eq!(nfa.accept_states.len(), 1);
        
        let start_state = &nfa.states[&nfa.start_state];
        assert_eq!(start_state.transitions.len(), 1);
        
        match &start_state.transitions[0].0 {
            Transition::Char(ch) => assert_eq!(*ch, 'a'),
            _ => panic!("Expected character transition"),
        }
    }
    
    #[test]
    fn test_char_class_nfa() {
        let class = CharClass::builtin(BuiltinClass::Digit);
        let nfa = Nfa::from_char_class(class);
        assert_eq!(nfa.states.len(), 2);
        assert_eq!(nfa.accept_states.len(), 1);
    }
    
    #[test]
    fn test_concatenation_nfa() {
        let first = Nfa::from_char('a');
        let second = Nfa::from_char('b');
        let concat = Nfa::concat(first, second);
        
        // Should have 4 states total (2 from each) but first's accept
        // is no longer accepting
        assert_eq!(concat.states.len(), 4);
        assert_eq!(concat.accept_states.len(), 1);
    }
    
    #[test]
    fn test_alternation_nfa() {
        let first = Nfa::from_char('a');
        let second = Nfa::from_char('b');
        let alt = Nfa::alternate(first, second).unwrap();
        
        // Should have 6 states: new start, new accept, 2 from each sub-NFA
        assert_eq!(alt.states.len(), 6);
        assert_eq!(alt.accept_states.len(), 1);
    }
    
    #[test]
    fn test_kleene_star_nfa() {
        let inner = Nfa::from_char('a');
        let star = Nfa::kleene_star(inner);
        
        // Should have 4 states: new start, new accept, 2 from inner
        assert_eq!(star.states.len(), 4);
        assert_eq!(star.accept_states.len(), 1);
    }
}