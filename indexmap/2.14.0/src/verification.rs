//! Creusot-facing ordered-sequence model for positional `IndexMap` and
//! `IndexSet` operations.

extern crate alloc;

use alloc::vec::Vec;
#[cfg(feature = "std")]
use std::collections::hash_map::RandomState;

#[allow(unused_imports)]
use creusot_std::prelude::{
    Int, Invariant, Seq, View, ensures, invariant, logic, pearlite, requires, snapshot, trusted,
    variant,
};

#[logic(open)]
#[allow(unreachable_pub)]
pub fn prefix_len(length: Int, requested: Int) -> Int {
    pearlite! { if requested < length { requested } else { length } }
}

/// A map's verification view is its key-value sequence in iteration order.
#[cfg(feature = "std")]
pub struct IndexMap<K, V, S = RandomState> {
    entries: Vec<(K, V)>,
    hash_builder: S,
}

/// A map's verification view is its key-value sequence in iteration order.
#[cfg(not(feature = "std"))]
pub struct IndexMap<K, V, S> {
    entries: Vec<(K, V)>,
    hash_builder: S,
}

impl<K, V, S> View for IndexMap<K, V, S> {
    type ViewTy = Seq<(K, V)>;

    #[logic]
    fn view(self) -> Self::ViewTy {
        pearlite! { self.entries@ }
    }
}

impl<K, V, S> Invariant for IndexMap<K, V, S> {
    #[logic(prophetic)]
    fn invariant(self) -> bool {
        pearlite! { 0 <= self@.len() && self@.len() <= usize::MAX@ }
    }
}

#[cfg(feature = "std")]
impl<K, V> IndexMap<K, V> {
    /// Create an empty map. The random hasher construction is the only trusted
    /// step; the contract exposes no property of its seed.
    #[trusted]
    #[ensures(result@ == Seq::empty())]
    pub fn new() -> Self {
        Self::with_hasher(RandomState::new())
    }

    /// Create an empty map with requested capacity.
    #[trusted]
    #[ensures(result@ == Seq::empty())]
    pub fn with_capacity(capacity: usize) -> Self {
        Self::with_capacity_and_hasher(capacity, RandomState::new())
    }
}

impl<K, V, S> IndexMap<K, V, S> {
    /// Create an empty map with a caller-provided hasher.
    #[ensures(result@ == Seq::empty())]
    pub fn with_hasher(hash_builder: S) -> Self {
        Self {
            entries: Vec::new(),
            hash_builder,
        }
    }

    /// Create an empty map with capacity and a caller-provided hasher.
    #[ensures(result@ == Seq::empty())]
    pub fn with_capacity_and_hasher(capacity: usize, hash_builder: S) -> Self {
        Self {
            entries: Vec::with_capacity(capacity),
            hash_builder,
        }
    }

    /// Return the number of key-value pairs.
    #[ensures(result@ == self@.len())]
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Return whether the map has no key-value pairs.
    #[ensures(result == (self@.len() == 0))]
    pub fn is_empty(&self) -> bool {
        self.entries.len() == 0
    }

    /// Return a capacity no smaller than the current length.
    #[ensures(result@ >= self@.len())]
    pub fn capacity(&self) -> usize {
        self.entries.capacity()
    }

    /// Return the caller-provided hash builder.
    pub fn hasher(&self) -> &S {
        &self.hash_builder
    }

    /// Remove every entry.
    #[ensures((^self)@ == Seq::empty())]
    pub fn clear(&mut self) {
        self.entries.clear();
    }

    /// Keep exactly the requested prefix, or the whole sequence when shorter.
    #[ensures((^self)@ == self@.subsequence(0, prefix_len(self@.len(), len@)))]
    pub fn truncate(&mut self, len: usize) {
        let old = snapshot!(self@);
        let target = if len < self.entries.len() {
            len
        } else {
            self.entries.len()
        };
        #[invariant(target@ == prefix_len(old.len(), len@))]
        #[invariant(target@ <= self@.len() && self@.len() <= old.len())]
        #[invariant(self@ == old.subsequence(0, self@.len()))]
        #[variant(self@.len() - target@)]
        while self.entries.len() > target {
            self.entries.pop();
        }
    }

    /// Remove and return the last entry.
    #[ensures(match result {
        Some(entry) => self@ == (^self)@.push_back(entry),
        None => self@.len() == 0 && (^self)@ == self@,
    })]
    pub fn pop(&mut self) -> Option<(K, V)> {
        self.entries.pop()
    }

    /// Remove one entry while shifting later entries down.
    #[ensures(match result {
        Some(entry) => index@ < self@.len()
            && entry == self@[index@]
            && (^self)@ == self@.removed(index@),
        None => index@ >= self@.len() && (^self)@ == self@,
    })]
    pub fn shift_remove_index(&mut self, index: usize) -> Option<(K, V)> {
        if index < self.entries.len() {
            Some(self.entries.remove(index))
        } else {
            None
        }
    }

    /// Exchange two valid positions without changing any other entry.
    #[requires(a@ < self@.len() && b@ < self@.len())]
    #[ensures(self@.exchange((^self)@, a@, b@))]
    pub fn swap_indices(&mut self, a: usize, b: usize) {
        self.entries.swap(a, b);
    }

    /// Reserve additional storage without changing ordered contents.
    #[ensures((^self)@ == self@)]
    pub fn reserve(&mut self, additional: usize) {
        self.entries.reserve(additional);
    }

    /// Reserve exact additional storage without changing ordered contents.
    #[ensures((^self)@ == self@)]
    pub fn reserve_exact(&mut self, additional: usize) {
        self.entries.reserve_exact(additional);
    }

    /// Shrink storage without changing ordered contents.
    #[ensures((^self)@ == self@)]
    pub fn shrink_to_fit(&mut self) {
        self.entries.shrink_to_fit();
    }

    /// Shrink storage to a lower bound without changing ordered contents.
    #[ensures((^self)@ == self@)]
    pub fn shrink_to(&mut self, min_capacity: usize) {
        self.entries.shrink_to(min_capacity);
    }
}

impl<K, V, S: Default> Default for IndexMap<K, V, S> {
    #[ensures(result@ == Seq::empty())]
    fn default() -> Self {
        Self::with_hasher(S::default())
    }
}

/// A set's verification view is its value sequence in iteration order.
#[cfg(feature = "std")]
pub struct IndexSet<T, S = RandomState> {
    entries: Vec<T>,
    hash_builder: S,
}

/// A set's verification view is its value sequence in iteration order.
#[cfg(not(feature = "std"))]
pub struct IndexSet<T, S> {
    entries: Vec<T>,
    hash_builder: S,
}

impl<T, S> View for IndexSet<T, S> {
    type ViewTy = Seq<T>;

    #[logic]
    fn view(self) -> Self::ViewTy {
        pearlite! { self.entries@ }
    }
}

impl<T, S> Invariant for IndexSet<T, S> {
    #[logic(prophetic)]
    fn invariant(self) -> bool {
        pearlite! { 0 <= self@.len() && self@.len() <= usize::MAX@ }
    }
}

#[cfg(feature = "std")]
impl<T> IndexSet<T> {
    /// Create an empty set. Only random hasher creation is trusted.
    #[trusted]
    #[ensures(result@ == Seq::empty())]
    pub fn new() -> Self {
        Self::with_hasher(RandomState::new())
    }

    /// Create an empty set with requested capacity.
    #[trusted]
    #[ensures(result@ == Seq::empty())]
    pub fn with_capacity(capacity: usize) -> Self {
        Self::with_capacity_and_hasher(capacity, RandomState::new())
    }
}

impl<T, S> IndexSet<T, S> {
    /// Create an empty set with a caller-provided hasher.
    #[ensures(result@ == Seq::empty())]
    pub fn with_hasher(hash_builder: S) -> Self {
        Self {
            entries: Vec::new(),
            hash_builder,
        }
    }

    /// Create an empty set with capacity and a caller-provided hasher.
    #[ensures(result@ == Seq::empty())]
    pub fn with_capacity_and_hasher(capacity: usize, hash_builder: S) -> Self {
        Self {
            entries: Vec::with_capacity(capacity),
            hash_builder,
        }
    }

    /// Return the number of values.
    #[ensures(result@ == self@.len())]
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Return whether the set is empty.
    #[ensures(result == (self@.len() == 0))]
    pub fn is_empty(&self) -> bool {
        self.entries.len() == 0
    }

    /// Return a capacity no smaller than the current length.
    #[ensures(result@ >= self@.len())]
    pub fn capacity(&self) -> usize {
        self.entries.capacity()
    }

    /// Return the caller-provided hash builder.
    pub fn hasher(&self) -> &S {
        &self.hash_builder
    }

    /// Remove every value.
    #[ensures((^self)@ == Seq::empty())]
    pub fn clear(&mut self) {
        self.entries.clear();
    }

    /// Keep exactly the requested prefix, or the whole sequence when shorter.
    #[ensures((^self)@ == self@.subsequence(0, prefix_len(self@.len(), len@)))]
    pub fn truncate(&mut self, len: usize) {
        let old = snapshot!(self@);
        let target = if len < self.entries.len() {
            len
        } else {
            self.entries.len()
        };
        #[invariant(target@ == prefix_len(old.len(), len@))]
        #[invariant(target@ <= self@.len() && self@.len() <= old.len())]
        #[invariant(self@ == old.subsequence(0, self@.len()))]
        #[variant(self@.len() - target@)]
        while self.entries.len() > target {
            self.entries.pop();
        }
    }

    /// Remove and return the last value.
    #[ensures(match result {
        Some(value) => self@ == (^self)@.push_back(value),
        None => self@.len() == 0 && (^self)@ == self@,
    })]
    pub fn pop(&mut self) -> Option<T> {
        self.entries.pop()
    }

    /// Remove one value while shifting later values down.
    #[ensures(match result {
        Some(value) => index@ < self@.len()
            && value == self@[index@]
            && (^self)@ == self@.removed(index@),
        None => index@ >= self@.len() && (^self)@ == self@,
    })]
    pub fn shift_remove_index(&mut self, index: usize) -> Option<T> {
        if index < self.entries.len() {
            Some(self.entries.remove(index))
        } else {
            None
        }
    }

    /// Exchange two valid positions without changing any other value.
    #[requires(a@ < self@.len() && b@ < self@.len())]
    #[ensures(self@.exchange((^self)@, a@, b@))]
    pub fn swap_indices(&mut self, a: usize, b: usize) {
        self.entries.swap(a, b);
    }

    /// Reserve additional storage without changing ordered contents.
    #[ensures((^self)@ == self@)]
    pub fn reserve(&mut self, additional: usize) {
        self.entries.reserve(additional);
    }

    /// Reserve exact additional storage without changing ordered contents.
    #[ensures((^self)@ == self@)]
    pub fn reserve_exact(&mut self, additional: usize) {
        self.entries.reserve_exact(additional);
    }

    /// Shrink storage without changing ordered contents.
    #[ensures((^self)@ == self@)]
    pub fn shrink_to_fit(&mut self) {
        self.entries.shrink_to_fit();
    }

    /// Shrink storage to a lower bound without changing ordered contents.
    #[ensures((^self)@ == self@)]
    pub fn shrink_to(&mut self, min_capacity: usize) {
        self.entries.shrink_to(min_capacity);
    }
}

impl<T, S: Default> Default for IndexSet<T, S> {
    #[ensures(result@ == Seq::empty())]
    fn default() -> Self {
        Self::with_hasher(S::default())
    }
}
