//! CRDT-based Consistency System - Phase 5 Stage 3
//!
//! This module implements Conflict-free Replicated Data Types (CRDTs) for maintaining
//! distributed state consistency without requiring coordination between nodes.
//!
//! Key Features:
//! - State-based CRDTs (CvRDTs) for eventual consistency
//! - Operation-based CRDTs (CmRDTs) for efficient delta synchronization
//! - Vector clocks for causality tracking
//! - Automatic conflict resolution
//! - Optimized synchronization protocols
//! - Integration with distributed execution engine

use super::{
    ConcurrencyError,
    byzantine_node_discovery::NodeIdentity,
    distributed_execution_engine::{NodeCapabilities, NodeId, TaskId},
};
use crate::diagnostics::{Error, Result};
use crate::eval::Value;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap, HashSet, VecDeque};
use std::sync::{Arc, RwLock};
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tokio::sync::{RwLock as AsyncRwLock, mpsc};
use tokio::time::interval;

/// Maximum number of delta operations to retain
const MAX_DELTA_HISTORY: usize = 1000;
/// Synchronization interval between nodes
const SYNC_INTERVAL: Duration = Duration::from_secs(5);
/// Maximum causality chain length
const MAX_CAUSALITY_CHAIN: usize = 1000;

/// Vector clock for tracking causality in distributed systems
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct VectorClock {
    /// Clock values for each node
    pub clocks: BTreeMap<NodeId, u64>,
}

impl VectorClock {
    /// Creates a new empty vector clock
    pub fn new() -> Self {
        Self {
            clocks: BTreeMap::new(),
        }
    }

    /// Creates a vector clock with single node entry
    pub fn with_node(node_id: NodeId, value: u64) -> Self {
        let mut clocks = BTreeMap::new();
        clocks.insert(node_id, value);
        Self { clocks }
    }

    /// Increments the clock for a specific node
    pub fn increment(&mut self, node_id: NodeId) {
        let current = self.clocks.get(&node_id).unwrap_or(&0);
        self.clocks.insert(node_id, current + 1);
    }

    /// Updates this clock with information from another clock (merge)
    pub fn merge(&mut self, other: &VectorClock) {
        for (&node_id, &other_value) in &other.clocks {
            let current_value = self.clocks.get(&node_id).unwrap_or(&0);
            self.clocks
                .insert(node_id, (*current_value).max(other_value));
        }
    }

    /// Checks if this clock happens before another clock
    pub fn happens_before(&self, other: &VectorClock) -> bool {
        let mut any_less = false;

        // Check all nodes in this clock
        for (&node_id, &this_value) in &self.clocks {
            let other_value = other.clocks.get(&node_id).unwrap_or(&0);
            if this_value > *other_value {
                return false;
            }
            if this_value < *other_value {
                any_less = true;
            }
        }

        // Check nodes only in other clock
        for (&node_id, &other_value) in &other.clocks {
            if !self.clocks.contains_key(&node_id) && other_value > 0 {
                any_less = true;
            }
        }

        any_less
    }

    /// Checks if this clock is concurrent with another clock
    pub fn concurrent(&self, other: &VectorClock) -> bool {
        !self.happens_before(other) && !other.happens_before(self) && self != other
    }

    /// Gets the clock value for a specific node
    pub fn get(&self, node_id: NodeId) -> u64 {
        self.clocks.get(&node_id).copied().unwrap_or(0)
    }

    /// Gets the sum of all clock values
    pub fn sum(&self) -> u64 {
        self.clocks.values().sum()
    }
}

impl Default for VectorClock {
    fn default() -> Self {
        Self::new()
    }
}

/// CRDT operation types
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum CrdtOperation {
    /// Add element to a G-Set (grow-only set)
    /// Struct Field documentation
    /// Struct Field documentation
    GSetAdd { element: String },
    /// Add element to 2P-Set (add-remove set)
    /// Struct Field documentation
    /// Struct Field documentation
    TwoPSetAdd { element: String },
    /// Remove element from 2P-Set  
    /// Struct Field documentation
    /// Struct Field documentation
    TwoPSetRemove { element: String },
    /// Increment a G-Counter (grow-only counter)
    /// Struct Field documentation
    /// Struct Field documentation
    /// Struct Field documentation
    /// Struct Field documentation
    GCounterIncrement { node_id: NodeId, delta: u64 },
    /// Increment a PN-Counter (increment-decrement counter)
    /// Struct Field documentation
    /// Struct Field documentation
    /// Struct Field documentation
    /// Struct Field documentation
    PNCounterIncrement { node_id: NodeId, delta: u64 },
    /// Decrement a PN-Counter
    /// Struct Field documentation
    /// Struct Field documentation
    /// Struct Field documentation
    /// Struct Field documentation
    PNCounterDecrement { node_id: NodeId, delta: u64 },
    /// Set value in an LWW-Register (last-writer-wins register)
    /// Struct Field documentation
    /// Struct Field documentation
    /// Struct Field documentation
    /// Struct Field documentation
    LWWRegisterSet { value: String, timestamp: u64 },
    /// Add entry to OR-Set (observed-remove set)
    /// Struct Field documentation
    /// Struct Field documentation
    /// Struct Field documentation
    /// Struct Field documentation
    ORSetAdd { element: String, tag: String },
    /// Remove entry from OR-Set
    /// Struct Field documentation
    /// Struct Field documentation
    /// Struct Field documentation
    /// Struct Field documentation
    ORSetRemove { element: String, tag: String },
    /// Update value in multi-value register
    /// Struct Field documentation
    /// Struct Field documentation
    MVRegisterSet { value: String },
}

impl CrdtOperation {
    /// Checks if this operation commutes with another operation
    pub fn commutes_with(&self, other: &CrdtOperation) -> bool {
        match (self, other) {
            // G-Set operations always commute
            (CrdtOperation::GSetAdd { .. }, CrdtOperation::GSetAdd { .. }) => true,

            // 2P-Set operations commute
            (CrdtOperation::TwoPSetAdd { .. }, _) => true,
            (CrdtOperation::TwoPSetRemove { .. }, _) => true,
            (_, CrdtOperation::TwoPSetAdd { .. }) => true,
            (_, CrdtOperation::TwoPSetRemove { .. }) => true,

            // Counter operations commute
            (CrdtOperation::GCounterIncrement { .. }, CrdtOperation::GCounterIncrement { .. }) => {
                true
            }
            (CrdtOperation::PNCounterIncrement { .. }, _) => true,
            (CrdtOperation::PNCounterDecrement { .. }, _) => true,
            (_, CrdtOperation::PNCounterIncrement { .. }) => true,
            (_, CrdtOperation::PNCounterDecrement { .. }) => true,

            // LWW-Register operations don't commute (order matters)
            (CrdtOperation::LWWRegisterSet { .. }, CrdtOperation::LWWRegisterSet { .. }) => false,

            // OR-Set operations commute
            (CrdtOperation::ORSetAdd { .. }, _) => true,
            (CrdtOperation::ORSetRemove { .. }, _) => true,
            (_, CrdtOperation::ORSetAdd { .. }) => true,
            (_, CrdtOperation::ORSetRemove { .. }) => true,

            // MV-Register operations don't commute
            (CrdtOperation::MVRegisterSet { .. }, CrdtOperation::MVRegisterSet { .. }) => false,

            _ => false,
        }
    }
}

/// CRDT delta for efficient synchronization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrdtDelta {
    /// Unique delta identifier
    pub delta_id: String,
    /// Node that generated this delta
    pub origin_node: NodeId,
    /// Vector clock at time of delta creation
    pub vector_clock: VectorClock,
    /// CRDT operation
    pub operation: CrdtOperation,
    /// Timestamp when delta was created
    pub timestamp: SystemTime,
    /// CRDT identifier this delta applies to
    pub crdt_id: String,
}

impl CrdtDelta {
    /// Creates a new CRDT delta
    pub fn new(
        origin_node: NodeId,
        vector_clock: VectorClock,
        operation: CrdtOperation,
        crdt_id: String,
    ) -> Self {
        Self {
            delta_id: format!("delta-{}-{}", origin_node, vector_clock.sum()),
            origin_node,
            vector_clock,
            operation,
            timestamp: SystemTime::now(),
            crdt_id,
        }
    }
}

/// G-Set: Grow-only Set CRDT
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GSet {
    /// Set elements
    elements: HashSet<String>,
}

impl GSet {
    /// Creates a new G-Set
    pub fn new() -> Self {
        Self {
            elements: HashSet::new(),
        }
    }

    /// Adds an element to the set
    pub fn add(&mut self, element: String) {
        self.elements.insert(element);
    }

    /// Checks if element is in the set
    pub fn contains(&self, element: &str) -> bool {
        self.elements.contains(element)
    }

    /// Gets all elements in the set
    pub fn elements(&self) -> Vec<String> {
        self.elements.iter().cloned().collect()
    }

    /// Merges with another G-Set
    pub fn merge(&mut self, other: &GSet) {
        for element in &other.elements {
            self.elements.insert(element.clone());
        }
    }

    /// Gets the size of the set
    pub fn len(&self) -> usize {
        self.elements.len()
    }

    /// Checks if the set is empty
    pub fn is_empty(&self) -> bool {
        self.elements.is_empty()
    }
}

impl Default for GSet {
    fn default() -> Self {
        Self::new()
    }
}

/// 2P-Set: Two-Phase Set CRDT (Add-Remove Set)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TwoPSet {
    /// Added elements
    added: HashSet<String>,
    /// Removed elements
    removed: HashSet<String>,
}

impl TwoPSet {
    /// Creates a new 2P-Set
    pub fn new() -> Self {
        Self {
            added: HashSet::new(),
            removed: HashSet::new(),
        }
    }

    /// Adds an element to the set
    pub fn add(&mut self, element: String) {
        self.added.insert(element);
    }

    /// Removes an element from the set
    pub fn remove(&mut self, element: String) {
        if self.added.contains(&element) {
            self.removed.insert(element);
        }
    }

    /// Checks if element is in the set
    pub fn contains(&self, element: &str) -> bool {
        self.added.contains(element) && !self.removed.contains(element)
    }

    /// Gets all elements in the set
    pub fn elements(&self) -> Vec<String> {
        self.added.difference(&self.removed).cloned().collect()
    }

    /// Merges with another 2P-Set
    pub fn merge(&mut self, other: &TwoPSet) {
        for element in &other.added {
            self.added.insert(element.clone());
        }
        for element in &other.removed {
            self.removed.insert(element.clone());
        }
    }

    /// Gets the size of the set
    pub fn len(&self) -> usize {
        self.added.difference(&self.removed).count()
    }

    /// Checks if the set is empty
    pub fn is_empty(&self) -> bool {
        self.added.difference(&self.removed).next().is_none()
    }
}

impl Default for TwoPSet {
    fn default() -> Self {
        Self::new()
    }
}

/// G-Counter: Grow-only Counter CRDT
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GCounter {
    /// Counter values per node
    counters: HashMap<NodeId, u64>,
}

impl GCounter {
    /// Creates a new G-Counter
    pub fn new() -> Self {
        Self {
            counters: HashMap::new(),
        }
    }

    /// Increments the counter for a specific node
    pub fn increment(&mut self, node_id: NodeId, delta: u64) {
        let current = self.counters.get(&node_id).unwrap_or(&0);
        self.counters.insert(node_id, current + delta);
    }

    /// Gets the total counter value
    pub fn value(&self) -> u64 {
        self.counters.values().sum()
    }

    /// Gets the counter value for a specific node
    pub fn get_node_value(&self, node_id: NodeId) -> u64 {
        self.counters.get(&node_id).copied().unwrap_or(0)
    }

    /// Merges with another G-Counter
    pub fn merge(&mut self, other: &GCounter) {
        for (&node_id, &other_value) in &other.counters {
            let current_value = self.counters.get(&node_id).unwrap_or(&0);
            self.counters
                .insert(node_id, (*current_value).max(other_value));
        }
    }
}

impl Default for GCounter {
    fn default() -> Self {
        Self::new()
    }
}

/// PN-Counter: Increment-Decrement Counter CRDT
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PNCounter {
    /// Positive increments
    positive: GCounter,
    /// Negative decrements
    negative: GCounter,
}

impl PNCounter {
    /// Creates a new PN-Counter
    pub fn new() -> Self {
        Self {
            positive: GCounter::new(),
            negative: GCounter::new(),
        }
    }

    /// Increments the counter
    pub fn increment(&mut self, node_id: NodeId, delta: u64) {
        self.positive.increment(node_id, delta);
    }

    /// Decrements the counter
    pub fn decrement(&mut self, node_id: NodeId, delta: u64) {
        self.negative.increment(node_id, delta);
    }

    /// Gets the total counter value
    pub fn value(&self) -> i64 {
        self.positive.value() as i64 - self.negative.value() as i64
    }

    /// Merges with another PN-Counter
    pub fn merge(&mut self, other: &PNCounter) {
        self.positive.merge(&other.positive);
        self.negative.merge(&other.negative);
    }
}

impl Default for PNCounter {
    fn default() -> Self {
        Self::new()
    }
}

/// LWW-Register: Last-Writer-Wins Register CRDT
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LWWRegister {
    /// Current value
    value: String,
    /// Timestamp of last write
    timestamp: u64,
}

impl LWWRegister {
    /// Creates a new LWW-Register
    pub fn new() -> Self {
        Self {
            value: String::new(),
            timestamp: 0,
        }
    }

    /// Sets the register value with timestamp
    pub fn set(&mut self, value: String, timestamp: u64) {
        if timestamp > self.timestamp {
            self.value = value;
            self.timestamp = timestamp;
        }
    }

    /// Gets the current value
    pub fn value(&self) -> &str {
        &self.value
    }

    /// Gets the timestamp
    pub fn timestamp(&self) -> u64 {
        self.timestamp
    }

    /// Merges with another LWW-Register
    pub fn merge(&mut self, other: &LWWRegister) {
        if other.timestamp > self.timestamp {
            self.value = other.value.clone();
            self.timestamp = other.timestamp;
        }
    }
}

impl Default for LWWRegister {
    fn default() -> Self {
        Self::new()
    }
}

/// OR-Set: Observed-Remove Set CRDT
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ORSet {
    /// Elements with their unique tags
    elements: HashMap<String, HashSet<String>>,
    /// Removed element-tag pairs
    removed: HashSet<(String, String)>,
}

impl ORSet {
    /// Creates a new OR-Set
    pub fn new() -> Self {
        Self {
            elements: HashMap::new(),
            removed: HashSet::new(),
        }
    }

    /// Adds an element with a unique tag
    pub fn add(&mut self, element: String, tag: String) {
        self.elements.entry(element).or_default().insert(tag);
    }

    /// Removes an element with all its observed tags
    pub fn remove(&mut self, element: String) {
        if let Some(tags) = self.elements.get(&element) {
            for tag in tags.clone() {
                self.removed.insert((element.clone(), tag));
            }
        }
    }

    /// Removes a specific element-tag pair
    pub fn remove_tag(&mut self, element: String, tag: String) {
        self.removed.insert((element, tag));
    }

    /// Checks if element is in the set
    pub fn contains(&self, element: &str) -> bool {
        if let Some(tags) = self.elements.get(element) {
            tags.iter()
                .any(|tag| !self.removed.contains(&(element.to_string(), tag.clone())))
        } else {
            false
        }
    }

    /// Gets all elements in the set
    pub fn elements(&self) -> Vec<String> {
        let mut result = Vec::new();
        for (element, tags) in &self.elements {
            let has_unremoved_tag = tags
                .iter()
                .any(|tag| !self.removed.contains(&(element.clone(), tag.clone())));
            if has_unremoved_tag {
                result.push(element.clone());
            }
        }
        result
    }

    /// Merges with another OR-Set
    pub fn merge(&mut self, other: &ORSet) {
        // Merge elements
        for (element, tags) in &other.elements {
            let entry = self.elements.entry(element.clone()).or_default();
            for tag in tags {
                entry.insert(tag.clone());
            }
        }

        // Merge removed set
        for removed_pair in &other.removed {
            self.removed.insert(removed_pair.clone());
        }
    }

    /// Gets the size of the set
    pub fn len(&self) -> usize {
        self.elements().len()
    }

    /// Checks if the set is empty
    pub fn is_empty(&self) -> bool {
        self.elements().is_empty()
    }
}

impl Default for ORSet {
    fn default() -> Self {
        Self::new()
    }
}

/// Multi-Value Register CRDT
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MVRegister {
    /// Values with their vector clocks
    values: HashMap<String, VectorClock>,
}

impl MVRegister {
    /// Creates a new MV-Register
    pub fn new() -> Self {
        Self {
            values: HashMap::new(),
        }
    }

    /// Sets a value with vector clock
    pub fn set(&mut self, value: String, vector_clock: VectorClock) {
        // Remove any values that are dominated by this vector clock
        self.values
            .retain(|_, vc| !vc.happens_before(&vector_clock));

        // Add the new value
        self.values.insert(value, vector_clock);
    }

    /// Gets all concurrent values
    pub fn values(&self) -> Vec<String> {
        self.values.keys().cloned().collect()
    }

    /// Merges with another MV-Register
    pub fn merge(&mut self, other: &MVRegister) {
        for (value, vector_clock) in &other.values {
            // Check if this value is dominated by any existing value
            let dominated = self
                .values
                .values()
                .any(|vc| vector_clock.happens_before(vc));

            if !dominated {
                // Remove any values dominated by this one
                self.values.retain(|_, vc| !vc.happens_before(vector_clock));
                self.values.insert(value.clone(), vector_clock.clone());
            }
        }
    }

    /// Checks if the register is empty
    pub fn is_empty(&self) -> bool {
        self.values.is_empty()
    }

    /// Gets the number of concurrent values
    pub fn len(&self) -> usize {
        self.len()
    }
}

impl Default for MVRegister {
    fn default() -> Self {
        Self::new()
    }
}

/// CRDT data types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CrdtData {
    /// Grow-only set
    GSet(GSet),
    /// Two-phase set
    TwoPSet(TwoPSet),
    /// Grow-only counter
    GCounter(GCounter),
    /// Increment-decrement counter
    PNCounter(PNCounter),
    /// Last-writer-wins register
    LWWRegister(LWWRegister),
    /// Observed-remove set
    ORSet(ORSet),
    /// Multi-value register
    MVRegister(MVRegister),
}

impl CrdtData {
    /// Applies an operation to the CRDT
    pub fn apply_operation(&mut self, operation: &CrdtOperation) -> Result<()> {
        match (self, operation) {
            (CrdtData::GSet(gset), CrdtOperation::GSetAdd { element }) => {
                gset.add(element.clone());
            }
            (CrdtData::TwoPSet(twopset), CrdtOperation::TwoPSetAdd { element }) => {
                twopset.add(element.clone());
            }
            (CrdtData::TwoPSet(twopset), CrdtOperation::TwoPSetRemove { element }) => {
                twopset.remove(element.clone());
            }
            (CrdtData::GCounter(gcounter), CrdtOperation::GCounterIncrement { node_id, delta }) => {
                gcounter.increment(*node_id, *delta);
            }
            (
                CrdtData::PNCounter(pncounter),
                CrdtOperation::PNCounterIncrement { node_id, delta },
            ) => {
                pncounter.increment(*node_id, *delta);
            }
            (
                CrdtData::PNCounter(pncounter),
                CrdtOperation::PNCounterDecrement { node_id, delta },
            ) => {
                pncounter.decrement(*node_id, *delta);
            }
            (CrdtData::LWWRegister(lww), CrdtOperation::LWWRegisterSet { value, timestamp }) => {
                lww.set(value.clone(), *timestamp);
            }
            (CrdtData::ORSet(orset), CrdtOperation::ORSetAdd { element, tag }) => {
                orset.add(element.clone(), tag.clone());
            }
            (CrdtData::ORSet(orset), CrdtOperation::ORSetRemove { element, tag }) => {
                orset.remove_tag(element.clone(), tag.clone());
            }
            (CrdtData::MVRegister(mvregister), CrdtOperation::MVRegisterSet { value }) => {
                // For simplicity, use current time as vector clock
                let mut vc = VectorClock::new();
                let timestamp = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_millis() as u64;
                vc.clocks.insert(NodeId::new(), timestamp); // Placeholder node
                mvregister.set(value.clone(), vc);
            }
            _ => {
                return Err(Box::new(Error::runtime_error(
                    "Operation not compatible with CRDT type".to_string(),
                    None,
                )));
            }
        }
        Ok(())
    }

    /// Merges with another CRDT of the same type
    pub fn merge(&mut self, other: &CrdtData) -> Result<()> {
        match (self, other) {
            (CrdtData::GSet(a), CrdtData::GSet(b)) => a.merge(b),
            (CrdtData::TwoPSet(a), CrdtData::TwoPSet(b)) => a.merge(b),
            (CrdtData::GCounter(a), CrdtData::GCounter(b)) => a.merge(b),
            (CrdtData::PNCounter(a), CrdtData::PNCounter(b)) => a.merge(b),
            (CrdtData::LWWRegister(a), CrdtData::LWWRegister(b)) => a.merge(b),
            (CrdtData::ORSet(a), CrdtData::ORSet(b)) => a.merge(b),
            (CrdtData::MVRegister(a), CrdtData::MVRegister(b)) => a.merge(b),
            _ => {
                return Err(Box::new(Error::runtime_error(
                    "Cannot merge CRDTs of different types".to_string(),
                    None,
                )));
            }
        }
        Ok(())
    }

    /// Converts CRDT data to a readable value
    pub fn to_value(&self) -> Value {
        match self {
            CrdtData::GSet(gset) => {
                let elements: Vec<Value> = gset
                    .elements()
                    .into_iter()
                    .map(|s| Value::Literal(crate::ast::Literal::String(Box::new(s))))
                    .collect();
                Value::Vector(std::rc::Rc::new(std::cell::RefCell::new(elements)))
            }
            CrdtData::TwoPSet(twopset) => {
                let elements: Vec<Value> = twopset
                    .elements()
                    .into_iter()
                    .map(|s| Value::Literal(crate::ast::Literal::String(Box::new(s))))
                    .collect();
                Value::Vector(std::rc::Rc::new(std::cell::RefCell::new(elements)))
            }
            CrdtData::GCounter(gcounter) => {
                Value::Literal(crate::ast::Literal::ExactInteger(gcounter.value() as i64))
            }
            CrdtData::PNCounter(pncounter) => {
                Value::Literal(crate::ast::Literal::ExactInteger(pncounter.value()))
            }
            CrdtData::LWWRegister(lww) => Value::Literal(crate::ast::Literal::String(Box::new(
                lww.value().to_string(),
            ))),
            CrdtData::ORSet(orset) => {
                let elements: Vec<Value> = orset
                    .elements()
                    .into_iter()
                    .map(|s| Value::Literal(crate::ast::Literal::String(Box::new(s))))
                    .collect();
                Value::Vector(std::rc::Rc::new(std::cell::RefCell::new(elements)))
            }
            CrdtData::MVRegister(mvregister) => {
                let values: Vec<Value> = mvregister
                    .values()
                    .into_iter()
                    .map(|s| Value::Literal(crate::ast::Literal::String(Box::new(s))))
                    .collect();
                Value::Vector(std::rc::Rc::new(std::cell::RefCell::new(values)))
            }
        }
    }
}

/// CRDT instance with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrdtInstance {
    /// Unique CRDT identifier
    pub id: String,
    /// CRDT data
    pub data: CrdtData,
    /// Current vector clock
    pub vector_clock: VectorClock,
    /// Node that owns this CRDT instance
    pub owner_node: NodeId,
    /// Last update timestamp
    pub last_updated: SystemTime,
    /// Version number for debugging
    pub version: u64,
}

impl CrdtInstance {
    /// Creates a new CRDT instance
    pub fn new(id: String, data: CrdtData, owner_node: NodeId) -> Self {
        Self {
            id,
            data,
            vector_clock: VectorClock::new(),
            owner_node,
            last_updated: SystemTime::now(),
            version: 1,
        }
    }

    /// Applies an operation to this CRDT
    pub fn apply_operation(
        &mut self,
        operation: &CrdtOperation,
        origin_node: NodeId,
    ) -> Result<()> {
        self.data.apply_operation(operation)?;
        self.vector_clock.increment(origin_node);
        self.last_updated = SystemTime::now();
        self.version += 1;
        Ok(())
    }

    /// Merges with another CRDT instance
    pub fn merge(&mut self, other: &CrdtInstance) -> Result<()> {
        if self.id != other.id {
            return Err(Box::new(Error::runtime_error(
                "Cannot merge CRDTs with different IDs".to_string(),
                None,
            )));
        }

        self.data.merge(&other.data)?;
        self.vector_clock.merge(&other.vector_clock);
        self.last_updated = self.last_updated.max(other.last_updated);
        self.version = self.version.max(other.version);

        Ok(())
    }

    /// Checks if this instance is newer than another
    pub fn is_newer_than(&self, other: &CrdtInstance) -> bool {
        self.vector_clock.happens_before(&other.vector_clock)
    }

    /// Gets the CRDT data as a readable value
    pub fn value(&self) -> Value {
        self.data.to_value()
    }
}

/// CRDT synchronization manager
#[derive(Debug)]
pub struct CrdtSynchronizationManager {
    /// Local CRDT instances
    crdts: Arc<RwLock<HashMap<String, CrdtInstance>>>,
    /// Delta operation history
    delta_history: Arc<RwLock<VecDeque<CrdtDelta>>>,
    /// Known peer nodes
    peer_nodes: Arc<RwLock<HashMap<NodeId, NodeIdentity>>>,
    /// Local node ID
    local_node: NodeId,
    /// Synchronization channels
    sync_tx: mpsc::UnboundedSender<CrdtDelta>,
    sync_rx: Arc<tokio::sync::Mutex<mpsc::UnboundedReceiver<CrdtDelta>>>,
}

impl CrdtSynchronizationManager {
    /// Creates a new CRDT synchronization manager
    pub fn new(local_node: NodeId) -> Self {
        let (sync_tx, sync_rx) = mpsc::unbounded_channel();

        Self {
            crdts: Arc::new(RwLock::new(HashMap::new())),
            delta_history: Arc::new(RwLock::new(VecDeque::new())),
            peer_nodes: Arc::new(RwLock::new(HashMap::new())),
            local_node,
            sync_tx,
            sync_rx: Arc::new(tokio::sync::Mutex::new(sync_rx)),
        }
    }

    /// Starts the synchronization process
    pub async fn start(&self) -> Result<()> {
        self.start_delta_processing().await?;
        self.start_periodic_sync().await?;
        Ok(())
    }

    /// Starts delta operation processing
    async fn start_delta_processing(&self) -> Result<()> {
        let crdts = self.crdts.clone();
        let delta_history = self.delta_history.clone();
        let sync_rx = self.sync_rx.clone();

        tokio::spawn(async move {
            let mut rx = sync_rx.lock().await;

            while let Some(delta) = rx.recv().await {
                // Apply delta to local CRDT
                if let Err(e) = Self::apply_delta_internal(&crdts, &delta_history, delta).await {
                    eprintln!("Failed to apply delta: {}", e);
                }
            }
        });

        Ok(())
    }

    /// Applies a delta operation
    async fn apply_delta_internal(
        crdts: &Arc<RwLock<HashMap<String, CrdtInstance>>>,
        delta_history: &Arc<RwLock<VecDeque<CrdtDelta>>>,
        delta: CrdtDelta,
    ) -> Result<()> {
        // Check if we already processed this delta
        {
            let history = delta_history.read().map_err(|_| {
                Error::runtime_error("Failed to acquire delta history lock".to_string(), None)
            })?;

            if history.iter().any(|d| d.delta_id == delta.delta_id) {
                return Ok(()); // Already processed
            }
        }

        // Apply operation to CRDT
        {
            let mut crdts_guard = crdts.write().map_err(|_| {
                Error::runtime_error("Failed to acquire CRDTs lock".to_string(), None)
            })?;

            if let Some(crdt) = crdts_guard.get_mut(&delta.crdt_id) {
                crdt.apply_operation(&delta.operation, delta.origin_node)?;
            }
        }

        // Add to delta history
        {
            let mut history = delta_history.write().map_err(|_| {
                Error::runtime_error("Failed to acquire delta history lock".to_string(), None)
            })?;

            history.push_back(delta);

            // Keep history size under control
            if history.len() > MAX_DELTA_HISTORY {
                history.pop_front();
            }
        }

        Ok(())
    }

    /// Starts periodic synchronization with peers
    async fn start_periodic_sync(&self) -> Result<()> {
        let crdts = self.crdts.clone();
        let peer_nodes = self.peer_nodes.clone();
        let local_node = self.local_node;

        tokio::spawn(async move {
            let mut sync_interval = interval(SYNC_INTERVAL);

            loop {
                sync_interval.tick().await;

                // Get peer list
                let peers: Vec<NodeId> = {
                    let peers_guard = peer_nodes.read().unwrap();
                    peers_guard.keys().copied().collect()
                };

                // Synchronize with each peer
                for peer_id in peers {
                    if let Err(e) = Self::synchronize_with_peer(&crdts, local_node, peer_id).await {
                        eprintln!("Synchronization with peer {} failed: {}", peer_id, e);
                    }
                }
            }
        });

        Ok(())
    }

    /// Synchronizes with a specific peer
    async fn synchronize_with_peer(
        crdts: &Arc<RwLock<HashMap<String, CrdtInstance>>>,
        local_node: NodeId,
        peer_id: NodeId,
    ) -> Result<()> {
        // In a real implementation, this would involve network communication
        // For now, we'll simulate synchronization
        println!("Synchronizing with peer {}", peer_id);

        // Get all local CRDTs
        let local_crdts: Vec<CrdtInstance> = {
            let crdts_guard = crdts.read().map_err(|_| {
                Error::runtime_error("Failed to acquire CRDTs lock".to_string(), None)
            })?;
            crdts_guard.values().cloned().collect()
        };

        // In practice, you would:
        // 1. Send vector clocks to peer
        // 2. Receive peer's vector clocks
        // 3. Exchange only newer deltas
        // 4. Apply received deltas

        println!(
            "Synchronized {} CRDTs with peer {}",
            local_crdts.len(),
            peer_id
        );
        Ok(())
    }

    /// Creates a new CRDT
    pub fn create_crdt(&self, id: String, crdt_type: CrdtData) -> Result<()> {
        let mut crdts = self
            .crdts
            .write()
            .map_err(|_| Error::runtime_error("Failed to acquire CRDTs lock".to_string(), None))?;

        if crdts.contains_key(&id) {
            return Err(Box::new(Error::runtime_error(
                format!("CRDT with ID '{}' already exists", id),
                None,
            )));
        }

        let crdt = CrdtInstance::new(id.clone(), crdt_type, self.local_node);
        crdts.insert(id, crdt);

        Ok(())
    }

    /// Applies an operation to a CRDT
    pub fn apply_operation(&self, crdt_id: &str, operation: CrdtOperation) -> Result<()> {
        // Create delta
        let vector_clock = {
            let crdts = self.crdts.read().map_err(|_| {
                Error::runtime_error("Failed to acquire CRDTs lock".to_string(), None)
            })?;

            crdts
                .get(crdt_id)
                .map(|crdt| crdt.vector_clock.clone())
                .unwrap_or_default()
        };

        let delta = CrdtDelta::new(
            self.local_node,
            vector_clock,
            operation,
            crdt_id.to_string(),
        );

        // Send delta for processing
        self.sync_tx.send(delta).map_err(|e| {
            Box::new(Error::runtime_error(
                format!("Failed to send delta: {}", e),
                None,
            ))
        })?;

        Ok(())
    }

    /// Gets a CRDT instance
    pub fn get_crdt(&self, id: &str) -> Result<Option<CrdtInstance>> {
        let crdts = self
            .crdts
            .read()
            .map_err(|_| Error::runtime_error("Failed to acquire CRDTs lock".to_string(), None))?;

        Ok(crdts.get(id).cloned())
    }

    /// Lists all CRDT IDs
    pub fn list_crdts(&self) -> Result<Vec<String>> {
        let crdts = self
            .crdts
            .read()
            .map_err(|_| Error::runtime_error("Failed to acquire CRDTs lock".to_string(), None))?;

        Ok(crdts.keys().cloned().collect())
    }

    /// Adds a peer node for synchronization
    pub fn add_peer(&self, peer: NodeIdentity) -> Result<()> {
        let mut peers = self.peer_nodes.write().map_err(|_| {
            Error::runtime_error("Failed to acquire peer nodes lock".to_string(), None)
        })?;

        peers.insert(peer.node_id, peer);
        Ok(())
    }

    /// Removes a peer node
    pub fn remove_peer(&self, peer_id: NodeId) -> Result<()> {
        let mut peers = self.peer_nodes.write().map_err(|_| {
            Error::runtime_error("Failed to acquire peer nodes lock".to_string(), None)
        })?;

        peers.remove(&peer_id);
        Ok(())
    }

    /// Gets synchronization statistics
    pub fn get_statistics(&self) -> Result<CrdtSyncStatistics> {
        let crdt_count = {
            let crdts = self.crdts.read().map_err(|_| {
                Error::runtime_error("Failed to acquire CRDTs lock".to_string(), None)
            })?;
            crdts.len()
        };

        let delta_count = {
            let history = self.delta_history.read().map_err(|_| {
                Error::runtime_error("Failed to acquire delta history lock".to_string(), None)
            })?;
            history.len()
        };

        let peer_count = {
            let peers = self.peer_nodes.read().map_err(|_| {
                Error::runtime_error("Failed to acquire peer nodes lock".to_string(), None)
            })?;
            peers.len()
        };

        Ok(CrdtSyncStatistics {
            total_crdts: crdt_count as u64,
            total_deltas: delta_count as u64,
            peer_nodes: peer_count as u64,
            sync_operations: 0, // TODO: Track sync operations
        })
    }
}

/// CRDT synchronization statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrdtSyncStatistics {
    /// Total number of CRDTs managed
    pub total_crdts: u64,
    /// Total number of delta operations
    pub total_deltas: u64,
    /// Number of peer nodes
    pub peer_nodes: u64,
    /// Number of synchronization operations performed
    pub sync_operations: u64,
}

/// Main CRDT-based consistency system
#[derive(Debug)]
pub struct CrdtConsistencySystem {
    /// CRDT synchronization manager
    sync_manager: Arc<CrdtSynchronizationManager>,
    /// Local node identifier
    local_node: NodeId,
}

impl CrdtConsistencySystem {
    /// Creates a new CRDT consistency system
    pub fn new(local_node: NodeId) -> Self {
        let sync_manager = Arc::new(CrdtSynchronizationManager::new(local_node));

        Self {
            sync_manager,
            local_node,
        }
    }

    /// Starts the consistency system
    pub async fn start(&self) -> Result<()> {
        self.sync_manager.start().await
    }

    /// Creates a new distributed counter
    pub fn create_counter(&self, id: String) -> Result<()> {
        self.sync_manager
            .create_crdt(id, CrdtData::GCounter(GCounter::new()))
    }

    /// Creates a new distributed set
    pub fn create_set(&self, id: String) -> Result<()> {
        self.sync_manager
            .create_crdt(id, CrdtData::GSet(GSet::new()))
    }

    /// Creates a new distributed register
    pub fn create_register(&self, id: String) -> Result<()> {
        self.sync_manager
            .create_crdt(id, CrdtData::LWWRegister(LWWRegister::new()))
    }

    /// Increments a distributed counter
    pub fn increment_counter(&self, counter_id: &str, delta: u64) -> Result<()> {
        let operation = CrdtOperation::GCounterIncrement {
            node_id: self.local_node,
            delta,
        };
        self.sync_manager.apply_operation(counter_id, operation)
    }

    /// Adds an element to a distributed set
    pub fn add_to_set(&self, set_id: &str, element: String) -> Result<()> {
        let operation = CrdtOperation::GSetAdd { element };
        self.sync_manager.apply_operation(set_id, operation)
    }

    /// Sets a value in a distributed register
    pub fn set_register(&self, register_id: &str, value: String) -> Result<()> {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;

        let operation = CrdtOperation::LWWRegisterSet { value, timestamp };
        self.sync_manager.apply_operation(register_id, operation)
    }

    /// Gets the value of a CRDT
    pub fn get_value(&self, crdt_id: &str) -> Result<Option<Value>> {
        self.sync_manager
            .get_crdt(crdt_id)
            .map(|opt| opt.map(|crdt| crdt.value()))
    }

    /// Adds a peer for synchronization
    pub fn add_peer(&self, peer: NodeIdentity) -> Result<()> {
        self.sync_manager.add_peer(peer)
    }

    /// Gets system statistics
    pub fn get_statistics(&self) -> Result<CrdtSyncStatistics> {
        self.sync_manager.get_statistics()
    }

    /// Lists all managed CRDTs
    pub fn list_crdts(&self) -> Result<Vec<String>> {
        self.sync_manager.list_crdts()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vector_clock() {
        let node1 = NodeId::new();
        let node2 = NodeId::new();

        let mut vc1 = VectorClock::new();
        let mut vc2 = VectorClock::new();

        vc1.increment(node1);
        assert_eq!(vc1.get(node1), 1);

        vc2.increment(node2);
        assert!(vc1.concurrent(&vc2));

        vc1.merge(&vc2);
        assert_eq!(vc1.get(node2), 1);
    }

    #[test]
    fn test_gset() {
        let mut set = GSet::new();

        set.add("element1".to_string());
        set.add("element2".to_string());

        assert!(set.contains("element1"));
        assert!(set.contains("element2"));
        assert!(!set.contains("element3"));
        assert_eq!(set.len(), 2);

        let mut other_set = GSet::new();
        other_set.add("element2".to_string());
        other_set.add("element3".to_string());

        set.merge(&other_set);
        assert_eq!(set.len(), 3);
        assert!(set.contains("element3"));
    }

    #[test]
    fn test_twop_set() {
        let mut set = TwoPSet::new();

        set.add("element1".to_string());
        assert!(set.contains("element1"));

        set.remove("element1".to_string());
        assert!(!set.contains("element1"));

        // Cannot re-add after remove
        set.add("element1".to_string());
        assert!(!set.contains("element1"));
    }

    #[test]
    fn test_gcounter() {
        let node1 = NodeId::new();
        let node2 = NodeId::new();

        let mut counter = GCounter::new();
        counter.increment(node1, 5);
        counter.increment(node2, 3);

        assert_eq!(counter.value(), 8);
        assert_eq!(counter.get_node_value(node1), 5);

        let mut other_counter = GCounter::new();
        other_counter.increment(node1, 3); // Should be ignored (3 < 5)
        other_counter.increment(node2, 7); // Should update (7 > 3)

        counter.merge(&other_counter);
        assert_eq!(counter.value(), 12); // 5 + 7
    }

    #[test]
    fn test_pn_counter() {
        let node1 = NodeId::new();
        let node2 = NodeId::new();

        let mut counter = PNCounter::new();
        counter.increment(node1, 10);
        counter.decrement(node2, 3);

        assert_eq!(counter.value(), 7);

        let mut other_counter = PNCounter::new();
        other_counter.increment(node1, 5);
        other_counter.increment(node2, 2);

        counter.merge(&other_counter);
        assert_eq!(counter.value(), 14); // 10 + 2 - 3 + 5 = 14
    }

    #[test]
    fn test_lww_register() {
        let mut register = LWWRegister::new();

        register.set("value1".to_string(), 100);
        assert_eq!(register.value(), "value1");

        register.set("value2".to_string(), 200);
        assert_eq!(register.value(), "value2");

        // Earlier timestamp should be ignored
        register.set("old_value".to_string(), 50);
        assert_eq!(register.value(), "value2");

        let mut other_register = LWWRegister::new();
        other_register.set("other_value".to_string(), 300);

        register.merge(&other_register);
        assert_eq!(register.value(), "other_value");
    }

    #[test]
    fn test_or_set() {
        let mut set = ORSet::new();

        set.add("element1".to_string(), "tag1".to_string());
        assert!(set.contains("element1"));

        set.add("element1".to_string(), "tag2".to_string());
        assert!(set.contains("element1"));

        set.remove_tag("element1".to_string(), "tag1".to_string());
        assert!(set.contains("element1")); // Still has tag2

        set.remove_tag("element1".to_string(), "tag2".to_string());
        assert!(!set.contains("element1"));
    }

    #[test]
    fn test_mv_register() {
        let node1 = NodeId::new();
        let node2 = NodeId::new();

        let mut register = MVRegister::new();

        let mut vc1 = VectorClock::with_node(node1, 1);
        let mut vc2 = VectorClock::with_node(node2, 1);

        register.set("value1".to_string(), vc1.clone());
        register.set("value2".to_string(), vc2.clone());

        // Both values should be present (concurrent)
        let values = register.values();
        assert_eq!(values.len(), 2);
        assert!(values.contains(&"value1".to_string()));
        assert!(values.contains(&"value2".to_string()));

        // Add a value that dominates others
        vc1.merge(&vc2);
        vc1.increment(node1);
        register.set("dominating_value".to_string(), vc1);

        let values = register.values();
        assert_eq!(values.len(), 1);
        assert!(values.contains(&"dominating_value".to_string()));
    }

    #[tokio::test]
    async fn test_crdt_synchronization_manager() {
        let node_id = NodeId::new();
        let manager = CrdtSynchronizationManager::new(node_id);

        manager.start().await.unwrap();

        // Create a counter
        manager
            .create_crdt("counter1".to_string(), CrdtData::GCounter(GCounter::new()))
            .unwrap();

        // Apply operations
        manager
            .apply_operation(
                "counter1",
                CrdtOperation::GCounterIncrement { node_id, delta: 5 },
            )
            .unwrap();

        // Small delay to process the delta
        tokio::time::sleep(Duration::from_millis(100)).await;

        // Check the value
        let crdt = manager.get_crdt("counter1").unwrap().unwrap();
        if let CrdtData::GCounter(counter) = crdt.data {
            assert_eq!(counter.value(), 5);
        } else {
            panic!("Expected GCounter");
        }
    }

    #[tokio::test]
    async fn test_crdt_consistency_system() {
        let node_id = NodeId::new();
        let system = CrdtConsistencySystem::new(node_id);

        system.start().await.unwrap();

        // Create and use a counter
        system.create_counter("test_counter".to_string()).unwrap();
        system.increment_counter("test_counter", 10).unwrap();

        // Small delay to process operations
        tokio::time::sleep(Duration::from_millis(100)).await;

        // Create and use a set
        system.create_set("test_set".to_string()).unwrap();
        system
            .add_to_set("test_set", "element1".to_string())
            .unwrap();

        tokio::time::sleep(Duration::from_millis(100)).await;

        let stats = system.get_statistics().unwrap();
        assert_eq!(stats.total_crdts, 2);
    }
}
