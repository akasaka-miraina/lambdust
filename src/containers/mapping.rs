//! HAMT-based immutable mapping implementation for SRFI-146
//!
//! This module provides a high-performance persistent mapping data structure
//! using Hash Array Mapped Trie (HAMT) for optimal structural sharing and
//! O(log₃₂ n) operations.
//!
//! ## Design Principles
//!
//! - **Immutability**: All operations return new mapping instances
//! - **Structural Sharing**: Unchanged parts of the trie are shared between instances
//! - **Performance**: O(log₃₂ n) operations with excellent practical performance
//! - **Memory Efficiency**: Path copying minimizes memory allocation
//!
//! ## Architecture
//!
//! The HAMT uses a branching factor of 32 (5 bits per level) with bitmapped
//! internal nodes to minimize memory usage. Each node uses a bitmap to track
//! which slots are occupied, allowing sparse representation.

use crate::containers::comparator::Comparator;
use crate::eval::value::Value;
use std::cmp::Ordering;
use std::sync::Arc;

/// Branching factor for HAMT (2^5 = 32)
const BRANCH_FACTOR: usize = 32;
/// Number of bits per level (log2(32) = 5)
const LEVEL_BITS: usize = 5;
/// Mask for extracting level bits (0x1F = 31)
const LEVEL_MASK: u32 = 0x1F;

/// Extract hash fragment for given level
#[inline]
fn hash_fragment(hash: u64, level: usize) -> usize {
    ((hash >> (level * LEVEL_BITS)) as usize) & (BRANCH_FACTOR - 1)
}

/// HAMT internal node with bitmapped sparse representation
#[derive(Clone, Debug)]
struct HamtNode {
    /// Bitmap indicating which slots are occupied (32 bits for 32 slots)
    bitmap: u32,
    /// Array of entries corresponding to set bits in bitmap
    entries: Vec<Entry>,
}

/// Entry in a HAMT node
#[derive(Clone, Debug)]
enum Entry {
    /// Leaf entry with key-value pair and hash
    Leaf { key: Value, value: Value, hash: u64 },
    /// Internal node reference
    Node(Arc<HamtNode>),
    /// Collision node for hash conflicts
    Collision(Vec<(Value, Value)>),
}

impl HamtNode {
    /// Creates an empty HAMT node
    pub fn empty() -> Self {
        Self {
            bitmap: 0,
            entries: Vec::new(),
        }
    }

    /// Checks if a slot is occupied
    #[inline]
    fn has_slot(&self, index: usize) -> bool {
        (self.bitmap & (1 << index)) != 0
    }

    /// Converts slot index to entry array index
    #[inline]
    fn slot_to_entry_index(&self, slot_index: usize) -> usize {
        (self.bitmap & ((1 << slot_index) - 1)).count_ones() as usize
    }

    /// Looks up a value in the HAMT
    pub fn lookup(
        &self,
        key: &Value,
        hash: u64,
        level: usize,
        comparator: &Comparator,
    ) -> Option<&Value> {
        let slot_index = hash_fragment(hash, level);

        if !self.has_slot(slot_index) {
            return None;
        }

        let entry_index = self.slot_to_entry_index(slot_index);
        match &self.entries[entry_index] {
            Entry::Leaf {
                key: leaf_key,
                value,
                hash: leaf_hash,
            } => {
                if *leaf_hash == hash && comparator.compare(key, leaf_key) == Ordering::Equal {
                    Some(value)
                } else {
                    None
                }
            }
            Entry::Node(child) => child.lookup(key, hash, level + 1, comparator),
            Entry::Collision(pairs) => pairs
                .iter()
                .find(|(k, _)| comparator.compare(key, k) == Ordering::Equal)
                .map(|(_, v)| v),
        }
    }

    /// Inserts or updates a key-value pair in the HAMT
    pub fn insert(
        self: Arc<Self>,
        key: Value,
        value: Value,
        hash: u64,
        level: usize,
        comparator: &Comparator,
    ) -> Arc<Self> {
        let slot_index = hash_fragment(hash, level);
        let entry_index = self.slot_to_entry_index(slot_index);

        if !self.has_slot(slot_index) {
            // Insert new entry
            self.insert_new_entry(slot_index, Entry::Leaf { key, value, hash })
        } else {
            // Update existing entry
            match &self.entries[entry_index] {
                Entry::Leaf {
                    key: existing_key,
                    value: existing_value,
                    hash: existing_hash,
                } => {
                    if *existing_hash == hash {
                        if comparator.compare(&key, existing_key) == Ordering::Equal {
                            // Replace existing value
                            self.replace_entry(entry_index, Entry::Leaf { key, value, hash })
                        } else {
                            // Hash collision - create collision node
                            let collision =
                                vec![(existing_key.clone(), existing_value.clone()), (key, value)];
                            self.replace_entry(entry_index, Entry::Collision(collision))
                        }
                    } else {
                        // Different hash - create internal node
                        let new_node = Arc::new(HamtNode::empty()).insert_new_entry(
                            hash_fragment(*existing_hash, level + 1),
                            Entry::Leaf {
                                key: existing_key.clone(),
                                value: existing_value.clone(),
                                hash: *existing_hash,
                            },
                        );

                        let updated_node = new_node.insert(key, value, hash, level + 1, comparator);

                        self.replace_entry(entry_index, Entry::Node(updated_node))
                    }
                }
                Entry::Node(child) => {
                    let updated_child =
                        child
                            .clone()
                            .insert(key, value, hash, level + 1, comparator);
                    self.replace_entry(entry_index, Entry::Node(updated_child))
                }
                Entry::Collision(pairs) => {
                    let mut new_pairs = pairs.clone();
                    if let Some(pos) = pairs
                        .iter()
                        .position(|(k, _)| comparator.compare(&key, k) == Ordering::Equal)
                    {
                        // Replace existing
                        new_pairs[pos] = (key, value);
                    } else {
                        // Add to collision
                        new_pairs.push((key, value));
                    }
                    self.replace_entry(entry_index, Entry::Collision(new_pairs))
                }
            }
        }
    }

    /// Removes a key from the HAMT
    pub fn remove(
        self: Arc<Self>,
        key: &Value,
        hash: u64,
        level: usize,
        comparator: &Comparator,
    ) -> Option<Arc<Self>> {
        let slot_index = hash_fragment(hash, level);

        if !self.has_slot(slot_index) {
            return Some(self);
        }

        let entry_index = self.slot_to_entry_index(slot_index);
        match &self.entries[entry_index] {
            Entry::Leaf {
                key: leaf_key,
                hash: leaf_hash,
                ..
            } => {
                if *leaf_hash == hash && comparator.compare(key, leaf_key) == Ordering::Equal {
                    Some(self.remove_entry(entry_index))
                } else {
                    Some(self)
                }
            }
            Entry::Node(child) => {
                if let Some(updated_child) = child.clone().remove(key, hash, level + 1, comparator)
                {
                    // Check if child became empty or has only one entry
                    if updated_child.entries.is_empty() {
                        Some(self.remove_entry(entry_index))
                    } else if updated_child.entries.len() == 1 {
                        // Collapse single-entry node
                        match &updated_child.entries[0] {
                            Entry::Leaf { .. } => Some(
                                self.replace_entry(entry_index, updated_child.entries[0].clone()),
                            ),
                            _ => Some(self.replace_entry(entry_index, Entry::Node(updated_child))),
                        }
                    } else {
                        Some(self.replace_entry(entry_index, Entry::Node(updated_child)))
                    }
                } else {
                    Some(self.remove_entry(entry_index))
                }
            }
            Entry::Collision(pairs) => {
                let new_pairs: Vec<_> = pairs
                    .iter()
                    .filter(|(k, _)| comparator.compare(key, k) != Ordering::Equal)
                    .cloned()
                    .collect();

                match new_pairs.len() {
                    0 => Some(self.remove_entry(entry_index)),
                    1 => {
                        let (k, v) = new_pairs[0].clone();
                        let hash = comparator.hash(&k);
                        Some(self.replace_entry(
                            entry_index,
                            Entry::Leaf {
                                key: k,
                                value: v,
                                hash,
                            },
                        ))
                    }
                    _ => Some(self.replace_entry(entry_index, Entry::Collision(new_pairs))),
                }
            }
        }
    }

    /// Inserts a new entry at the given slot index
    fn insert_new_entry(self: Arc<Self>, slot_index: usize, entry: Entry) -> Arc<Self> {
        let new_bitmap = self.bitmap | (1 << slot_index);
        let entry_index = self.slot_to_entry_index(slot_index);

        let mut new_entries = self.entries.clone();
        new_entries.insert(entry_index, entry);

        Arc::new(Self {
            bitmap: new_bitmap,
            entries: new_entries,
        })
    }

    /// Replaces an entry at the given entry index
    fn replace_entry(self: Arc<Self>, entry_index: usize, new_entry: Entry) -> Arc<Self> {
        let mut new_entries = self.entries.clone();
        new_entries[entry_index] = new_entry;

        Arc::new(Self {
            bitmap: self.bitmap,
            entries: new_entries,
        })
    }

    /// Removes an entry at the given entry index
    fn remove_entry(self: Arc<Self>, entry_index: usize) -> Arc<Self> {
        // Find which slot this entry corresponds to
        let mut remaining_bits = self.bitmap;
        let mut slot_index = 0;
        let mut current_entry_index = 0;

        while remaining_bits != 0 {
            if remaining_bits & 1 != 0 {
                if current_entry_index == entry_index {
                    break;
                }
                current_entry_index += 1;
            }
            slot_index += 1;
            remaining_bits >>= 1;
        }

        let new_bitmap = self.bitmap & !(1 << slot_index);
        let mut new_entries = self.entries.clone();
        new_entries.remove(entry_index);

        Arc::new(Self {
            bitmap: new_bitmap,
            entries: new_entries,
        })
    }

    /// Returns the number of entries in this node
    pub fn size(&self) -> usize {
        self.entries.len()
    }

    /// Checks if this node is empty
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

/// Persistent immutable mapping using HAMT
#[derive(Clone, Debug)]
pub struct PersistentMapping {
    /// Root HAMT node
    root: Arc<HamtNode>,
    /// Comparator for keys
    comparator: Comparator,
    /// Number of key-value pairs
    size: usize,
}

impl PersistentMapping {
    /// Creates a new empty mapping with the default comparator
    pub fn new() -> Self {
        Self {
            root: Arc::new(HamtNode::empty()),
            comparator: Comparator::with_default(),
            size: 0,
        }
    }

    /// Creates a new empty mapping with the specified comparator
    pub fn with_comparator(comparator: Comparator) -> Self {
        Self {
            root: Arc::new(HamtNode::empty()),
            comparator,
            size: 0,
        }
    }

    /// Returns the number of key-value pairs in the mapping
    pub fn size(&self) -> usize {
        self.size
    }

    /// Checks if the mapping is empty
    pub fn is_empty(&self) -> bool {
        self.size == 0
    }

    /// Looks up a value by key
    pub fn get(&self, key: &Value) -> Option<&Value> {
        let hash = self.comparator.hash(key);
        self.root.lookup(key, hash, 0, &self.comparator)
    }

    /// Checks if the mapping contains a key
    pub fn contains_key(&self, key: &Value) -> bool {
        self.get(key).is_some()
    }

    /// Returns a new mapping with the key-value pair added or updated
    pub fn insert(&self, key: Value, value: Value) -> Self {
        let hash = self.comparator.hash(&key);
        let old_size = self.size;
        let new_root = self
            .root
            .clone()
            .insert(key.clone(), value, hash, 0, &self.comparator);

        // Check if this was an insert or update
        let new_size = if self.contains_key(&key) {
            old_size // Update
        } else {
            old_size + 1 // Insert
        };

        Self {
            root: new_root,
            comparator: self.comparator.clone(),
            size: new_size,
        }
    }

    /// Returns a new mapping with the key removed
    pub fn remove(&self, key: &Value) -> Self {
        if !self.contains_key(key) {
            return self.clone();
        }

        let hash = self.comparator.hash(key);
        if let Some(new_root) = self.root.clone().remove(key, hash, 0, &self.comparator) {
            Self {
                root: new_root,
                comparator: self.comparator.clone(),
                size: self.size - 1,
            }
        } else {
            // Root was removed - empty mapping
            Self::with_comparator(self.comparator.clone())
        }
    }

    /// Returns the comparator used by this mapping
    pub fn comparator(&self) -> &Comparator {
        &self.comparator
    }
}

impl Default for PersistentMapping {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::eval::value::Value;

    #[test]
    fn test_empty_mapping() {
        let mapping = PersistentMapping::new();
        assert_eq!(mapping.size(), 0);
        assert!(mapping.is_empty());
        assert_eq!(mapping.get(&Value::integer(1)), None);
    }

    #[test]
    fn test_single_insertion() {
        let mapping = PersistentMapping::new();
        let key = Value::integer(42);
        let value = Value::string("hello");

        let mapping2 = mapping.insert(key.clone(), value.clone());
        assert_eq!(mapping2.size(), 1);
        assert!(!mapping2.is_empty());
        assert_eq!(mapping2.get(&key), Some(&value));

        // Original mapping should be unchanged
        assert_eq!(mapping.size(), 0);
    }

    #[test]
    fn test_multiple_insertions() {
        let mut mapping = PersistentMapping::new();

        for i in 0..10 {
            let key = Value::integer(i);
            let value = Value::string(&format!("value_{}", i));
            mapping = mapping.insert(key.clone(), value.clone());

            assert_eq!(mapping.size(), (i + 1) as usize);
            assert_eq!(mapping.get(&key), Some(&value));
        }

        // Verify all values are still there
        for i in 0..10 {
            let key = Value::integer(i);
            let expected_value = Value::string(&format!("value_{}", i));
            assert_eq!(mapping.get(&key), Some(&expected_value));
        }
    }

    #[test]
    fn test_key_update() {
        let mapping = PersistentMapping::new();
        let key = Value::integer(1);

        let mapping2 = mapping.insert(key.clone(), Value::string("first"));
        let mapping3 = mapping2.insert(key.clone(), Value::string("second"));

        assert_eq!(mapping2.size(), 1);
        assert_eq!(mapping3.size(), 1); // Size should stay the same
        assert_eq!(mapping3.get(&key), Some(&Value::string("second")));
    }

    #[test]
    fn test_removal() {
        let mapping = PersistentMapping::new();
        let key1 = Value::integer(1);
        let key2 = Value::integer(2);
        let value1 = Value::string("one");
        let value2 = Value::string("two");

        let mapping2 = mapping
            .insert(key1.clone(), value1.clone())
            .insert(key2.clone(), value2.clone());

        assert_eq!(mapping2.size(), 2);

        let mapping3 = mapping2.remove(&key1);
        assert_eq!(mapping3.size(), 1);
        assert_eq!(mapping3.get(&key1), None);
        assert_eq!(mapping3.get(&key2), Some(&value2));

        // Original should be unchanged
        assert_eq!(mapping2.size(), 2);
        assert_eq!(mapping2.get(&key1), Some(&value1));
    }

    #[test]
    fn test_hash_collisions() {
        // This test would require keys with known hash collisions
        // For now, we'll just test with different keys
        let mapping = PersistentMapping::new();

        let keys: Vec<Value> = (0..100).map(Value::integer).collect();
        let mut current = mapping;

        for (i, key) in keys.iter().enumerate() {
            let value = Value::string(&format!("value_{}", i));
            current = current.insert(key.clone(), value);
        }

        assert_eq!(current.size(), 100);

        for (i, key) in keys.iter().enumerate() {
            let expected_value = Value::string(&format!("value_{}", i));
            assert_eq!(current.get(key), Some(&expected_value));
        }
    }
}
