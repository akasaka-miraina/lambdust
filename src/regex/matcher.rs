//! NFA execution engine for pattern matching.
//!
//! This module implements the Thompson NFA simulation algorithm for efficient
//! regular expression matching. It uses the classical approach of maintaining
//! a set of active states and processing input characters one by one.
//!
//! ## Algorithm: NFA Simulation
//!
//! The matcher maintains two sets of states:
//! 1. **Current States**: States active before processing current character
//! 2. **Next States**: States active after processing current character
//!
//! For each input character:
//! 1. Start with current state set (initially epsilon closure of start state)
//! 2. For each state in current set, follow matching transitions
//! 3. Compute epsilon closure of resulting states → next state set
//! 4. Swap current ↔ next, continue with next character
//!
//! **Time Complexity**: O(nm) where n = text length, m = NFA size
//! **Space Complexity**: O(m) for state sets
//!
//! ## Features
//!
//! - **Epsilon Closure**: Handles epsilon transitions efficiently
//! - **Anchors**: Supports ^ (start) and $ (end) anchors  
//! - **Character Classes**: Efficient character class matching
//! - **Match Positions**: Tracks start/end positions of matches

use std::collections::HashSet;
use crate::regex::engine::{Nfa, NfaEngine, StateId, Transition};

/// Match result containing position and matched text.
#[derive(Debug, Clone, PartialEq)]
pub struct Match {
    /// Start position in input text (byte offset)
    pub start: usize,
    /// End position in input text (byte offset) 
    pub end: usize,
    /// Matched text slice
    text: String,
}

impl Match {
    /// Creates a new match result.
    pub fn new(start: usize, end: usize, text: String) -> Self {
        Self { start, end, text }
    }
    
    /// Returns the matched text as a string slice.
    pub fn as_str(&self) -> &str {
        &self.text
    }
    
    /// Returns the length of the match.
    pub fn len(&self) -> usize {
        self.end - self.start
    }
    
    /// Tests if the match is empty.
    pub fn is_empty(&self) -> bool {
        self.start == self.end
    }
}

/// Complete match result including potential captures.
#[derive(Debug, Clone)]
pub struct MatchResult {
    /// Primary match
    pub full_match: Match,
    /// Captured groups (for future extension)
    pub captures: Vec<Option<Match>>,
}

/// NFA matcher for executing pattern matching.
pub struct Matcher<'nfa> {
    /// NFA to execute
    nfa: &'nfa Nfa,
    /// Current active states
    current_states: HashSet<StateId>,
    /// Next active states  
    next_states: HashSet<StateId>,
    /// Whether we're at start of input
    at_start: bool,
}

impl<'nfa> Matcher<'nfa> {
    /// Creates a new matcher for the given NFA.
    pub fn new(engine: &'nfa NfaEngine) -> Self {
        Self {
            nfa: engine.nfa(),
            current_states: HashSet::new(),
            next_states: HashSet::new(),
            at_start: true,
        }
    }
    
    /// Finds the first match in the input text.
    pub fn find(&mut self, text: &str) -> Option<Match> {
        for start_pos in 0..=text.len() {
            if let Some(m) = self.find_at(text, start_pos) {
                return Some(m);
            }
        }
        None
    }
    
    /// Attempts to find a match starting at the specified position.
    pub fn find_at(&mut self, text: &str, start_pos: usize) -> Option<Match> {
        if start_pos > text.len() {
            return None;
        }
        
        // Initialize state sets
        self.current_states.clear();
        self.next_states.clear();
        self.at_start = start_pos == 0;
        
        // Start with epsilon closure of start state
        self.current_states.insert(self.nfa.start_state);
        Matcher::epsilon_closure_for_nfa(self.nfa, &mut self.current_states);
        
        // Check for immediate accept (empty match)
        if self.has_accept_state(&self.current_states) {
            return Some(Match::new(start_pos, start_pos, String::new()));
        }
        
        // Process each character
        let text_bytes = text.as_bytes();
        let mut pos = start_pos;
        
        while pos < text.len() {
            let ch = text_bytes[pos] as char; // Simplified: assume ASCII
            self.next_states.clear();
            
            // Process transitions for current character
            for &state_id in &self.current_states {
                if let Some(state) = self.nfa.states.get(&state_id) {
                    for (transition, target) in &state.transitions {
                        if self.transition_matches(transition, ch, pos, text.len()) {
                            self.next_states.insert(*target);
                        }
                    }
                }
            }
            
            // Compute epsilon closure of next states
            Matcher::epsilon_closure_for_nfa(self.nfa, &mut self.next_states);
            
            // Check for match
            if self.has_accept_state(&self.next_states) {
                let match_text = text[start_pos..=pos].to_string();
                return Some(Match::new(start_pos, pos + 1, match_text));
            }
            
            // If no active states, matching failed
            if self.next_states.is_empty() {
                break;
            }
            
            // Swap state sets for next iteration
            std::mem::swap(&mut self.current_states, &mut self.next_states);
            pos += 1;
            self.at_start = false;
        }
        
        // Handle end-of-input anchors
        if !self.current_states.is_empty() {
            self.next_states.clear();
            
            for &state_id in &self.current_states {
                if let Some(state) = self.nfa.states.get(&state_id) {
                    for (transition, target) in &state.transitions {
                        if matches!(transition, Transition::End) {
                            self.next_states.insert(*target);
                        }
                    }
                }
            }
            
            Matcher::epsilon_closure_for_nfa(self.nfa, &mut self.next_states);
            
            if self.has_accept_state(&self.next_states) {
                let match_text = text[start_pos..pos].to_string();
                return Some(Match::new(start_pos, pos, match_text));
            }
        }
        
        None
    }
    
    /// Tests if a transition matches the current character and context.
    fn transition_matches(&self, transition: &Transition, ch: char, pos: usize, text_len: usize) -> bool {
        match transition {
            Transition::Epsilon => false, // Handled separately
            Transition::Char(expected) => ch == *expected,
            Transition::CharClass(class) => class.matches(ch),
            Transition::Any => ch != '\n', // . doesn't match newline by default
            Transition::Start => pos == 0 || self.at_start,
            Transition::End => pos == text_len,
        }
    }
    
    /// Computes epsilon closure of a state set in-place.
    fn epsilon_closure(&self, states: &mut HashSet<StateId>) {
        Matcher::epsilon_closure_for_nfa(self.nfa, states);
    }
    
    /// Static helper for computing epsilon closure to avoid borrowing issues.
    fn epsilon_closure_for_nfa(nfa: &Nfa, states: &mut HashSet<StateId>) {
        let mut stack: Vec<StateId> = states.iter().copied().collect();
        
        while let Some(state_id) = stack.pop() {
            if let Some(state) = nfa.states.get(&state_id) {
                for (transition, target) in &state.transitions {
                    if matches!(transition, Transition::Epsilon)
                        && states.insert(*target) {
                            // New state added, explore it
                            stack.push(*target);
                        }
                }
            }
        }
    }
    
    /// Tests if any state in the set is an accept state.
    fn has_accept_state(&self, states: &HashSet<StateId>) -> bool {
        states.iter().any(|&state_id| {
            self.nfa.accept_states.contains(&state_id)
        })
    }
    
    /// Resets the matcher for reuse.
    pub fn reset(&mut self) {
        self.current_states.clear();
        self.next_states.clear();
        self.at_start = true;
    }
}

/// Iterator for finding all matches in text.
pub struct FindMatches<'m, 't> {
    matcher: &'m mut Matcher<'t>,
    text: &'t str,
    pos: usize,
}

impl<'m, 't> FindMatches<'m, 't> {
    /// Creates a new find iterator.
    pub fn new(matcher: &'m mut Matcher<'t>, text: &'t str) -> Self {
        Self {
            matcher,
            text,
            pos: 0,
        }
    }
}

impl<'m, 't> Iterator for FindMatches<'m, 't> {
    type Item = Match;
    
    fn next(&mut self) -> Option<Self::Item> {
        while self.pos <= self.text.len() {
            if let Some(m) = self.matcher.find_at(self.text, self.pos) {
                self.pos = m.end.max(self.pos + 1); // Ensure progress
                return Some(m);
            } else {
                self.pos += 1;
            }
        }
        None
    }
}

/// Helper function to create a match iterator.
pub fn find_iter<'t>(engine: &'t NfaEngine, text: &'t str) -> impl Iterator<Item = Match> + 't {
    let mut matcher = Matcher::new(engine);
    let mut pos = 0;
    
    std::iter::from_fn(move || {
        while pos <= text.len() {
            if let Some(m) = matcher.find_at(text, pos) {
                pos = m.end.max(pos + 1);
                return Some(m);
            } else {
                pos += 1;
            }
        }
        None
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::regex::parser::PatternParser;

    fn create_engine(pattern: &str) -> NfaEngine {
        let parsed = PatternParser::new(pattern).parse().unwrap();
        NfaEngine::from_pattern(&parsed).unwrap()
    }

    fn create_matcher(pattern: &str) -> (NfaEngine, Matcher) {
        let engine = create_engine(pattern);
        let matcher = Matcher::new(&engine);
        (engine, matcher)
    }

    #[test]
    fn test_simple_char_match() {
        let engine = create_engine("a");
        let mut matcher = Matcher::new(&engine);
        
        assert!(matcher.find("a").is_some());
        assert!(matcher.find("abc").is_some());
        assert!(matcher.find("bac").is_some());
        assert!(matcher.find("xyz").is_none());
    }
    
    #[test]
    fn test_concatenation_match() {
        let (_engine, mut matcher) = create_matcher("abc");
        
        let m = matcher.find("abc").unwrap();
        assert_eq!(m.start, 0);
        assert_eq!(m.end, 3);
        assert_eq!(m.as_str(), "abc");
        
        let m2 = matcher.find("xyzabc").unwrap();
        assert_eq!(m2.start, 3);
        assert_eq!(m2.end, 6);
        
        assert!(matcher.find("ab").is_none());
        assert!(matcher.find("acb").is_none());
    }
    
    #[test]
    fn test_alternation_match() {
        let (_engine, mut matcher) = create_matcher("a|b");
        
        assert!(matcher.find("a").is_some());
        assert!(matcher.find("b").is_some());
        assert!(matcher.find("c").is_none());
        
        let m = matcher.find("ba").unwrap();
        assert_eq!(m.as_str(), "b");
        assert_eq!(m.start, 0);
    }
    
    #[test]
    fn test_star_quantifier() {
        let (_engine, mut matcher) = create_matcher("a*");
        
        // Should match empty string
        let m = matcher.find("").unwrap();
        assert_eq!(m.start, 0);
        assert_eq!(m.end, 0);
        
        // Should match "a"
        let m2 = matcher.find("a").unwrap();
        assert_eq!(m2.as_str(), "a");
        
        // Should match "aaa"  
        let m3 = matcher.find("aaa").unwrap();
        assert_eq!(m3.as_str(), "aaa");
        
        // Should match at start of "baa"
        let m4 = matcher.find("baa").unwrap();
        assert_eq!(m4.start, 0);
        assert_eq!(m4.end, 0); // Empty match at start
    }
    
    #[test]
    fn test_plus_quantifier() {
        let (_engine, mut matcher) = create_matcher("a+");
        
        // Should not match empty string
        assert!(matcher.find("").is_none());
        
        // Should match "a"
        let m = matcher.find("a").unwrap();
        assert_eq!(m.as_str(), "a");
        
        // Should match "aaa"
        let m2 = matcher.find("aaa").unwrap();
        assert_eq!(m2.as_str(), "aaa");
        
        // Should not match at start of "baa"
        let m3 = matcher.find("baa").unwrap();
        assert_eq!(m3.start, 1); // Matches "aa" part
        assert_eq!(m3.as_str(), "aa");
    }
    
    #[test]
    fn test_question_quantifier() {
        let (_engine, mut matcher) = create_matcher("a?");
        
        // Should match empty string
        let m = matcher.find("").unwrap();
        assert_eq!(m.start, 0);
        assert_eq!(m.end, 0);
        
        // Should match "a"
        let m2 = matcher.find("a").unwrap();
        assert_eq!(m2.as_str(), "a");
        
        // Should match first "a" in "aa"
        let m3 = matcher.find("aa").unwrap();
        assert_eq!(m3.as_str(), "a");
        assert_eq!(m3.start, 0);
        assert_eq!(m3.end, 1);
    }
    
    #[test]
    fn test_any_char() {
        let (_engine, mut matcher) = create_matcher(".");
        
        assert!(matcher.find("a").is_some());
        assert!(matcher.find("1").is_some());
        assert!(matcher.find("@").is_some());
        assert!(matcher.find("").is_none());
        
        // Should not match newline by default
        assert!(matcher.find("\n").is_none());
    }
    
    #[test]
    fn test_character_class() {
        let (_engine, mut matcher) = create_matcher("[abc]");
        
        assert!(matcher.find("a").is_some());
        assert!(matcher.find("b").is_some());
        assert!(matcher.find("c").is_some());
        assert!(matcher.find("d").is_none());
        assert!(matcher.find("xay").is_some()); // Should find 'a'
    }
    
    #[test]
    fn test_digit_class() {
        let (_engine, mut matcher) = create_matcher(r"\d");
        
        assert!(matcher.find("5").is_some());
        assert!(matcher.find("0").is_some());
        assert!(matcher.find("9").is_some());
        assert!(matcher.find("a").is_none());
        
        let m = matcher.find("abc123").unwrap();
        assert_eq!(m.as_str(), "1");
        assert_eq!(m.start, 3);
    }
    
    #[test]
    fn test_word_class() {
        let (_engine, mut matcher) = create_matcher(r"\w");
        
        assert!(matcher.find("a").is_some());
        assert!(matcher.find("Z").is_some());
        assert!(matcher.find("5").is_some());
        assert!(matcher.find("_").is_some());
        assert!(matcher.find("@").is_none());
        assert!(matcher.find(" ").is_none());
    }
    
    #[test]
    fn test_complex_pattern() {
        let (_engine, mut matcher) = create_matcher(r"\d+\.\d*");
        
        let m = matcher.find("3.14").unwrap();
        assert_eq!(m.as_str(), "3.14");
        
        let m2 = matcher.find("42.").unwrap();
        assert_eq!(m2.as_str(), "42.");
        
        assert!(matcher.find("3").is_none());
        assert!(matcher.find(".14").is_none());
    }
    
    #[test] 
    fn test_find_positions() {
        let (_engine, mut matcher) = create_matcher("ab");
        
        let m = matcher.find("xyzab123").unwrap();
        assert_eq!(m.start, 3);
        assert_eq!(m.end, 5);
        assert_eq!(m.as_str(), "ab");
    }
}