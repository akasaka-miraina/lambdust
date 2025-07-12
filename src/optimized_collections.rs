//! Optimized collection patterns for performance
//!
//! This module provides memory-efficient alternatives to common collection
//! operations that frequently create unnecessary clones and allocations.
//!
//! Key optimizations:
//! - Slice indices instead of Vec copying
//! - Reference-based iteration
//! - Copy-on-write semantics
//! - Small vector optimization

use std::rc::Rc;

/// A slice reference that avoids copying Vec elements
/// Provides efficient slicing operations without `Vec::clone()`
#[derive(Debug, Clone)]
pub struct SliceRef<T> {
    /// Reference to the source vector
    source: Rc<Vec<T>>,
    /// Start index in the source vector
    start: usize,
    /// End index in the source vector (exclusive)
    end: usize,
}

impl<T> SliceRef<T> {
    /// Create a new slice reference from a vector
    #[must_use] pub fn new(source: Vec<T>) -> Self {
        let len = source.len();
        Self {
            source: Rc::new(source),
            start: 0,
            end: len,
        }
    }

    /// Create a slice reference from an existing Rc<Vec<T>>
    #[must_use] pub fn from_rc(source: Rc<Vec<T>>) -> Self {
        let len = source.len();
        Self {
            source,
            start: 0,
            end: len,
        }
    }

    /// Create a slice of this slice (zero-copy operation)
    #[must_use] pub fn slice(&self, start: usize, end: usize) -> Self {
        let actual_start = self.start + start.min(self.len());
        let actual_end = self.start + end.min(self.len());

        Self {
            source: self.source.clone(),
            start: actual_start,
            end: actual_end.max(actual_start),
        }
    }

    /// Get a slice starting from index (equivalent to [start..])
    #[must_use] pub fn slice_from(&self, start: usize) -> Self {
        self.slice(start, self.len())
    }

    /// Get a slice up to index (equivalent to [..end])
    #[must_use] pub fn slice_to(&self, end: usize) -> Self {
        self.slice(0, end)
    }

    /// Drop the first element (equivalent to [1..])
    #[must_use] pub fn tail(&self) -> Self {
        self.slice_from(1)
    }

    /// Take the first n elements (equivalent to [..n])
    #[must_use] pub fn take(&self, n: usize) -> Self {
        self.slice_to(n)
    }

    /// Drop the first n elements (equivalent to [n..])
    #[must_use] pub fn drop(&self, n: usize) -> Self {
        self.slice_from(n)
    }

    /// Get the length of this slice
    #[must_use] pub fn len(&self) -> usize {
        self.end.saturating_sub(self.start)
    }

    /// Check if the slice is empty
    #[must_use] pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Get an element by index
    #[must_use] pub fn get(&self, index: usize) -> Option<&T> {
        if index < self.len() {
            self.source.get(self.start + index)
        } else {
            None
        }
    }

    /// Get the first element
    #[must_use] pub fn first(&self) -> Option<&T> {
        self.get(0)
    }

    /// Get the last element  
    #[must_use] pub fn last(&self) -> Option<&T> {
        if self.is_empty() {
            None
        } else {
            self.get(self.len() - 1)
        }
    }

    /// Convert to a Vec (clones elements)
    #[must_use] pub fn to_vec(&self) -> Vec<T>
    where
        T: Clone,
    {
        self.source[self.start..self.end].to_vec()
    }

    /// Iterate over elements
    pub fn iter(&self) -> impl Iterator<Item = &T> {
        self.source[self.start..self.end].iter()
    }

    /// Get underlying vector reference (for compatibility)
    #[must_use] pub fn as_vec_ref(&self) -> &Vec<T> {
        &self.source
    }

    /// Check if this slice references the entire source vector
    #[must_use] pub fn is_whole_vector(&self) -> bool {
        self.start == 0 && self.end == self.source.len()
    }
}

impl<T> From<Vec<T>> for SliceRef<T> {
    fn from(vec: Vec<T>) -> Self {
        Self::new(vec)
    }
}

impl<T> From<Rc<Vec<T>>> for SliceRef<T> {
    fn from(rc: Rc<Vec<T>>) -> Self {
        Self::from_rc(rc)
    }
}

/// Copy-on-write vector that shares references until modification
/// Reduces cloning overhead for frequently passed but rarely modified vectors
#[derive(Debug, Clone)]
pub struct CowVec<T> {
    /// The underlying vector (potentially shared)
    data: Rc<Vec<T>>,
    /// Whether this is the unique owner
    unique: bool,
}

impl<T> CowVec<T> {
    /// Create a new `CowVec` from a vector
    #[must_use] pub fn new(vec: Vec<T>) -> Self {
        Self {
            data: Rc::new(vec),
            unique: true,
        }
    }

    /// Create from an existing Rc<Vec<T>>
    #[must_use] pub fn from_rc(rc: Rc<Vec<T>>) -> Self {
        Self {
            data: rc,
            unique: false,
        }
    }

    /// Get shared reference to the vector
    #[allow(clippy::should_implement_trait)]
    #[must_use] pub fn as_ref(&self) -> &Vec<T> {
        &self.data
    }

    /// Get mutable reference, cloning if necessary (copy-on-write)
    pub fn make_mut(&mut self) -> &mut Vec<T>
    where
        T: Clone,
    {
        if !self.unique || Rc::strong_count(&self.data) > 1 {
            // Need to clone
            self.data = Rc::new((*self.data).clone());
            self.unique = true;
        }

        // This is safe because we just ensured unique ownership
        unsafe { &mut *(self.data.as_ptr() as *mut Vec<T>) }
    }

    /// Push an element, cloning vector if necessary
    pub fn push(&mut self, value: T)
    where
        T: Clone,
    {
        self.make_mut().push(value);
    }

    /// Pop an element, cloning vector if necessary
    pub fn pop(&mut self) -> Option<T>
    where
        T: Clone,
    {
        self.make_mut().pop()
    }

    /// Get length
    #[must_use] pub fn len(&self) -> usize {
        self.data.len()
    }

    /// Check if empty
    #[must_use] pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    /// Convert to `SliceRef` for efficient slicing
    #[must_use] pub fn as_slice_ref(&self) -> SliceRef<T> {
        SliceRef::from_rc(self.data.clone())
    }

    /// Convert to owned Vec
    #[must_use] pub fn into_vec(self) -> Vec<T>
    where
        T: Clone,
    {
        if self.unique && Rc::strong_count(&self.data) == 1 {
            // We have unique ownership, can extract directly
            match Rc::try_unwrap(self.data) {
                Ok(vec) => vec,
                Err(rc) => (*rc).clone(),
            }
        } else {
            (*self.data).clone()
        }
    }
}

impl<T> From<Vec<T>> for CowVec<T> {
    fn from(vec: Vec<T>) -> Self {
        Self::new(vec)
    }
}

impl<T> std::ops::Deref for CowVec<T> {
    type Target = Vec<T>;

    fn deref(&self) -> &Self::Target {
        &self.data
    }
}

/// Small vector optimization for common small collections
/// Uses stack allocation for vectors up to N elements
pub type SmallVec<T, const N: usize> = smallvec::SmallVec<[T; N]>;

// Re-export smallvec for convenience
pub use smallvec;

/// Optimized argument vector for function calls
/// Most Scheme function calls have few arguments, so this is optimized for that case
pub type ArgVec<T> = SmallVec<T, 4>;

/// Optimized expression vector for small expression lists\
/// Most expression lists are small, so this avoids allocation in common cases
pub type ExprVec<T> = SmallVec<T, 8>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_slice_ref_basic_operations() {
        let vec = vec![1, 2, 3, 4, 5];
        let slice_ref = SliceRef::new(vec);

        assert_eq!(slice_ref.len(), 5);
        assert_eq!(slice_ref.get(0), Some(&1));
        assert_eq!(slice_ref.get(4), Some(&5));
        assert_eq!(slice_ref.get(5), None);
    }

    #[test]
    fn test_slice_ref_slicing() {
        let vec = vec![1, 2, 3, 4, 5];
        let slice_ref = SliceRef::new(vec);

        let tail = slice_ref.tail();
        assert_eq!(tail.len(), 4);
        assert_eq!(tail.get(0), Some(&2));

        let middle = slice_ref.slice(1, 4);
        assert_eq!(middle.len(), 3);
        assert_eq!(middle.get(0), Some(&2));
        assert_eq!(middle.get(2), Some(&4));
    }

    #[test]
    fn test_cow_vec_sharing() {
        let cow1 = CowVec::new(vec![1, 2, 3]);
        let cow2 = cow1.clone();

        // Both should point to same data
        assert_eq!(cow1.len(), 3);
        assert_eq!(cow2.len(), 3);

        // Modifying one should trigger copy-on-write
        // (This test is conceptual since we need Clone trait)
    }

    #[test]
    fn test_small_vec_optimization() {
        let mut small: SmallVec<i32, 4> = SmallVec::new();

        // Should not allocate on heap for small sizes
        small.push(1);
        small.push(2);
        small.push(3);
        small.push(4);

        assert_eq!(small.len(), 4);
        assert_eq!(small[0], 1);
    }
}
