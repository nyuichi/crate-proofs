//! Creusot-facing structural and control-byte model for hashbrown 0.17.1.
//!
//! The published SwissTable implementation remains untouched in the ordinary
//! crate target. This proof target isolates state transitions which do not
//! depend on hashing and a scalar form of the generic control-byte algorithm.

extern crate alloc;

use alloc::vec::Vec;
#[allow(unused_imports)]
use creusot_std::prelude::{DeepModel, Invariant, Seq, View, ensures, logic, pearlite, requires};

/// The encoded EMPTY control byte used by hashbrown.
pub const EMPTY: u8 = 0xff;
/// The encoded DELETED control byte used by hashbrown.
pub const DELETED: u8 = 0x80;

/// Whether a byte is a valid SwissTable control byte.
#[logic(open)]
pub fn valid_control(value: u8) -> bool {
    pearlite! { value == EMPTY || value == DELETED || value@ < 128 }
}

/// Scalar representative of one byte in the generic control group.
pub struct ControlByte(u8);

impl DeepModel for ControlByte {
    type DeepModelTy = u8;

    #[logic]
    fn deep_model(self) -> u8 {
        self.0
    }
}

impl Invariant for ControlByte {
    #[logic(prophetic)]
    fn invariant(self) -> bool {
        pearlite! { valid_control(self.deep_model()) }
    }
}

impl ControlByte {
    /// Wraps a valid encoded control byte.
    #[requires(valid_control(value))]
    #[ensures(result.deep_model() == value)]
    #[ensures(result.invariant())]
    pub fn from_encoded(value: u8) -> Self {
        Self(value)
    }

    /// Returns whether the byte denotes an empty bucket.
    ///
    /// This specifies the per-byte meaning of `Group::match_empty`.
    #[requires(self.invariant())]
    #[ensures(result == (self.deep_model() == EMPTY))]
    pub fn is_empty(&self) -> bool {
        self.0 == EMPTY
    }

    /// Returns whether the byte denotes an empty or deleted bucket.
    ///
    /// This specifies the per-byte meaning of `Group::match_empty_or_deleted`.
    #[requires(self.invariant())]
    #[ensures(result == (self.deep_model() == EMPTY || self.deep_model() == DELETED))]
    pub fn is_empty_or_deleted(&self) -> bool {
        self.0 == EMPTY || self.0 == DELETED
    }

    /// Returns whether the byte denotes a full bucket.
    #[requires(self.invariant())]
    #[ensures(result == (self.deep_model()@ < 128))]
    pub fn is_full(&self) -> bool {
        !self.is_empty_or_deleted()
    }

    /// Specifies hashbrown's generic control-byte conversion:
    /// EMPTY/DELETED become EMPTY and FULL becomes DELETED.
    #[requires(self.invariant())]
    #[ensures(result.deep_model() == if self.deep_model()@ < 128 { DELETED } else { EMPTY })]
    #[ensures(result.invariant())]
    pub fn convert_special_to_empty_and_full_to_deleted(self) -> Self {
        if self.0 < 128 {
            Self(DELETED)
        } else {
            Self(EMPTY)
        }
    }
}

/// A proof-facing table representation for hash-independent transitions.
pub struct HashTable<T> {
    values: Vec<T>,
}

impl<T> View for HashTable<T> {
    type ViewTy = Seq<T>;

    #[logic]
    fn view(self) -> Seq<T> {
        pearlite! { self.values@ }
    }
}

impl<T> Invariant for HashTable<T> {
    #[logic(prophetic)]
    fn invariant(self) -> bool {
        pearlite! { self@.len() <= usize::MAX@ }
    }
}

impl<T> HashTable<T> {
    /// Creates an empty table.
    #[ensures(result@ == Seq::empty())]
    #[ensures(result.invariant())]
    pub const fn new() -> Self {
        Self { values: Vec::new() }
    }

    /// Returns the exact logical element count.
    #[ensures(result@ == self@.len())]
    pub fn len(&self) -> usize {
        self.values.len()
    }

    /// Reports exact logical emptiness.
    #[ensures(result == (self@.len() == 0))]
    pub fn is_empty(&self) -> bool {
        self.values.len() == 0
    }

    /// Removes every logical element.
    #[ensures((^self)@ == Seq::empty())]
    #[ensures((^self).invariant())]
    pub fn clear(&mut self) {
        self.values.clear();
    }

    /// Adds an element under the caller's uniqueness obligation.
    ///
    /// The sequence order is proof-only; runtime `HashTable` iteration order is
    /// deliberately unspecified.
    #[requires(self.invariant())]
    #[requires(self@.len() < usize::MAX@)]
    #[ensures((^self)@ == self@.push_back(value))]
    #[ensures((^self).invariant())]
    pub fn insert_unique(&mut self, value: T) {
        self.values.push(value);
    }

    /// Removes and returns one logical element, if present.
    #[requires(self.invariant())]
    #[ensures(match result {
        Some(value) => self@ == (^self)@.push_back(value),
        None => (^self)@ == self@ && self@.len() == 0,
    })]
    #[ensures((^self).invariant())]
    pub fn pop(&mut self) -> Option<T> {
        self.values.pop()
    }
}

impl<T> Default for HashTable<T> {
    #[ensures(result@ == Seq::empty())]
    #[ensures(result.invariant())]
    fn default() -> Self {
        Self::new()
    }
}

/// Structural state model for the hash-independent `HashMap` surface.
pub struct HashMap<K, V, S> {
    entries: HashTable<(K, V)>,
    hash_builder: S,
}

impl<K, V, S> View for HashMap<K, V, S> {
    type ViewTy = Seq<(K, V)>;

    #[logic]
    fn view(self) -> Seq<(K, V)> {
        pearlite! { self.entries@ }
    }
}

impl<K, V, S> Invariant for HashMap<K, V, S> {
    #[logic(prophetic)]
    fn invariant(self) -> bool {
        pearlite! { self.entries.invariant() }
    }
}

impl<K, V, S> HashMap<K, V, S> {
    /// Creates an empty map with a caller-provided hash builder.
    #[ensures(result@ == Seq::empty())]
    #[ensures(result.invariant())]
    pub fn with_hasher(hash_builder: S) -> Self {
        Self {
            entries: HashTable::new(),
            hash_builder,
        }
    }

    /// Returns the exact logical entry count.
    #[ensures(result@ == self@.len())]
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Reports exact logical emptiness.
    #[ensures(result == (self@.len() == 0))]
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Returns the same caller-provided hash builder.
    pub fn hasher(&self) -> &S {
        &self.hash_builder
    }

    /// Removes every logical entry.
    #[ensures((^self)@ == Seq::empty())]
    #[ensures((^self).invariant())]
    pub fn clear(&mut self) {
        self.entries.clear();
    }
}

/// Structural state model for the hash-independent `HashSet` surface.
pub struct HashSet<T, S> {
    entries: HashTable<T>,
    hash_builder: S,
}

impl<T, S> View for HashSet<T, S> {
    type ViewTy = Seq<T>;

    #[logic]
    fn view(self) -> Seq<T> {
        pearlite! { self.entries@ }
    }
}

impl<T, S> Invariant for HashSet<T, S> {
    #[logic(prophetic)]
    fn invariant(self) -> bool {
        pearlite! { self.entries.invariant() }
    }
}

impl<T, S> HashSet<T, S> {
    /// Creates an empty set with a caller-provided hash builder.
    #[ensures(result@ == Seq::empty())]
    #[ensures(result.invariant())]
    pub fn with_hasher(hash_builder: S) -> Self {
        Self {
            entries: HashTable::new(),
            hash_builder,
        }
    }

    /// Returns the exact logical element count.
    #[ensures(result@ == self@.len())]
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Reports exact logical emptiness.
    #[ensures(result == (self@.len() == 0))]
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Returns the same caller-provided hash builder.
    pub fn hasher(&self) -> &S {
        &self.hash_builder
    }

    /// Removes every logical element.
    #[ensures((^self)@ == Seq::empty())]
    #[ensures((^self).invariant())]
    pub fn clear(&mut self) {
        self.entries.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::{DELETED, EMPTY};

    fn runtime_match_empty(value: u8) -> bool {
        value & (value << 1) & DELETED != 0
    }

    fn runtime_match_empty_or_deleted(value: u8) -> bool {
        value & DELETED != 0
    }

    fn runtime_convert(value: u8) -> u8 {
        let full = !value & DELETED;
        !full + (full >> 7)
    }

    #[test]
    fn scalar_spec_matches_generic_control_arithmetic_exhaustively() {
        for value in 0u8..=u8::MAX {
            let valid = value == EMPTY || value == DELETED || value < 128;
            if valid {
                assert_eq!(runtime_match_empty(value), value == EMPTY);
                assert_eq!(
                    runtime_match_empty_or_deleted(value),
                    value == EMPTY || value == DELETED
                );
                assert_eq!(
                    runtime_convert(value),
                    if value < 128 { DELETED } else { EMPTY }
                );
            }
        }
    }
}
