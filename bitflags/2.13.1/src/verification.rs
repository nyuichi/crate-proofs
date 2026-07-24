//! Creusot-facing representative expansion of the core `bitflags!` API.
//!
//! `bitflags` emits its implementation into downstream crates.  This type fixes
//! the storage to `u8` and the known-bit mask to three one-bit flags so the
//! generated state transitions can be proved without relying on downstream
//! macro expansion or a generic logical model for every integer type.

use core::ops::{
    BitAnd, BitAndAssign, BitOr, BitOrAssign, BitXor, BitXorAssign, Not, Sub, SubAssign,
};
use creusot_std::prelude::{ensures, logic, DeepModel};

pub struct ExampleFlags {
    bits: u8,
}

impl DeepModel for ExampleFlags {
    type DeepModelTy = u8;

    #[logic]
    fn deep_model(self) -> u8 {
        self.bits
    }
}

impl ExampleFlags {
    pub const A: Self = Self { bits: 0b001 };
    pub const B: Self = Self { bits: 0b010 };
    pub const C: Self = Self { bits: 0b100 };
    pub const KNOWN_BITS: u8 = 0b111;

    #[ensures(result.deep_model() == 0u8)]
    pub fn empty() -> Self {
        Self { bits: 0 }
    }

    #[ensures(result.deep_model() == Self::KNOWN_BITS)]
    pub fn all() -> Self {
        Self {
            bits: Self::KNOWN_BITS,
        }
    }

    #[ensures(result == self.deep_model())]
    pub fn bits(&self) -> u8 {
        self.bits
    }

    #[ensures(result.deep_model() == bits)]
    pub fn from_bits_retain(bits: u8) -> Self {
        Self { bits }
    }

    #[ensures(result.deep_model() == (bits & Self::KNOWN_BITS))]
    pub fn from_bits_truncate(bits: u8) -> Self {
        Self::from_bits_retain(bits & Self::KNOWN_BITS)
    }

    #[ensures(match result {
        Some(flags) => flags.deep_model() == bits && (bits & Self::KNOWN_BITS) == bits,
        None => (bits & Self::KNOWN_BITS) != bits,
    })]
    pub fn from_bits(bits: u8) -> Option<Self> {
        let truncated = Self::from_bits_truncate(bits);
        if truncated.bits() == bits {
            Some(truncated)
        } else {
            None
        }
    }

    #[ensures(result == (self.deep_model() & Self::KNOWN_BITS))]
    pub fn known_bits(&self) -> u8 {
        self.bits & Self::KNOWN_BITS
    }

    #[ensures(result == (self.deep_model() & !Self::KNOWN_BITS))]
    pub fn unknown_bits(&self) -> u8 {
        self.bits & !Self::KNOWN_BITS
    }

    #[ensures(result == ((self.deep_model() & !Self::KNOWN_BITS) != 0u8))]
    pub fn contains_unknown_bits(&self) -> bool {
        self.unknown_bits() != 0
    }

    #[ensures(result == (self.deep_model() == 0u8))]
    pub fn is_empty(&self) -> bool {
        self.bits == 0
    }

    #[ensures(result == ((Self::KNOWN_BITS | self.deep_model()) == self.deep_model()))]
    pub fn is_all(&self) -> bool {
        Self::KNOWN_BITS | self.bits == self.bits
    }

    #[ensures(result == ((self.deep_model() & other.deep_model()) != 0u8))]
    pub fn intersects(&self, other: Self) -> bool {
        self.bits & other.bits != 0
    }

    #[ensures(result == ((self.deep_model() & other.deep_model()) == other.deep_model()))]
    pub fn contains(&self, other: Self) -> bool {
        self.bits & other.bits == other.bits
    }

    #[ensures(result.deep_model() == (self.deep_model() & other.deep_model()))]
    pub fn intersection(self, other: Self) -> Self {
        Self::from_bits_retain(self.bits & other.bits)
    }

    #[ensures(result.deep_model() == (self.deep_model() | other.deep_model()))]
    pub fn union(self, other: Self) -> Self {
        Self::from_bits_retain(self.bits | other.bits)
    }

    #[ensures(result.deep_model() == (self.deep_model() & !other.deep_model()))]
    pub fn difference(self, other: Self) -> Self {
        Self::from_bits_retain(self.bits & !other.bits)
    }

    #[ensures(result.deep_model() == (self.deep_model() ^ other.deep_model()))]
    pub fn symmetric_difference(self, other: Self) -> Self {
        Self::from_bits_retain(self.bits ^ other.bits)
    }

    #[ensures(result.deep_model() == (!self.deep_model() & Self::KNOWN_BITS))]
    pub fn complement(self) -> Self {
        Self::from_bits_truncate(!self.bits)
    }

    #[ensures((^self).deep_model() == ((*self).deep_model() & Self::KNOWN_BITS))]
    pub fn truncate(&mut self) {
        *self = Self::from_bits_truncate(self.bits);
    }

    #[ensures((^self).deep_model() == ((*self).deep_model() | other.deep_model()))]
    pub fn insert(&mut self, other: Self) {
        *self = Self::from_bits_retain(self.bits).union(other);
    }

    #[ensures((^self).deep_model() == ((*self).deep_model() & !other.deep_model()))]
    pub fn remove(&mut self, other: Self) {
        *self = Self::from_bits_retain(self.bits).difference(other);
    }

    #[ensures((^self).deep_model() == ((*self).deep_model() ^ other.deep_model()))]
    pub fn toggle(&mut self, other: Self) {
        *self = Self::from_bits_retain(self.bits).symmetric_difference(other);
    }

    #[ensures((^self).deep_model() == if value {
        (*self).deep_model() | other.deep_model()
    } else {
        (*self).deep_model() & !other.deep_model()
    })]
    pub fn set(&mut self, other: Self, value: bool) {
        if value {
            self.insert(other);
        } else {
            self.remove(other);
        }
    }

    #[ensures((^self).deep_model() == 0u8)]
    pub fn clear(&mut self) {
        *self = Self::empty();
    }
}

impl BitAnd for ExampleFlags {
    type Output = Self;

    #[ensures(result.deep_model() == (self.deep_model() & rhs.deep_model()))]
    fn bitand(self, rhs: Self) -> Self {
        self.intersection(rhs)
    }
}

impl BitOr for ExampleFlags {
    type Output = Self;

    #[ensures(result.deep_model() == (self.deep_model() | rhs.deep_model()))]
    fn bitor(self, rhs: Self) -> Self {
        self.union(rhs)
    }
}

impl BitXor for ExampleFlags {
    type Output = Self;

    #[ensures(result.deep_model() == (self.deep_model() ^ rhs.deep_model()))]
    fn bitxor(self, rhs: Self) -> Self {
        self.symmetric_difference(rhs)
    }
}

impl Sub for ExampleFlags {
    type Output = Self;

    #[ensures(result.deep_model() == (self.deep_model() & !rhs.deep_model()))]
    fn sub(self, rhs: Self) -> Self {
        self.difference(rhs)
    }
}

impl Not for ExampleFlags {
    type Output = Self;

    #[ensures(result.deep_model() == (!self.deep_model() & Self::KNOWN_BITS))]
    fn not(self) -> Self {
        self.complement()
    }
}

impl BitAndAssign for ExampleFlags {
    #[ensures((^self).deep_model() == ((*self).deep_model() & rhs.deep_model()))]
    fn bitand_assign(&mut self, rhs: Self) {
        *self = Self::from_bits_retain(self.bits & rhs.bits);
    }
}

impl BitOrAssign for ExampleFlags {
    #[ensures((^self).deep_model() == ((*self).deep_model() | rhs.deep_model()))]
    fn bitor_assign(&mut self, rhs: Self) {
        *self = Self::from_bits_retain(self.bits | rhs.bits);
    }
}

impl BitXorAssign for ExampleFlags {
    #[ensures((^self).deep_model() == ((*self).deep_model() ^ rhs.deep_model()))]
    fn bitxor_assign(&mut self, rhs: Self) {
        *self = Self::from_bits_retain(self.bits ^ rhs.bits);
    }
}

impl SubAssign for ExampleFlags {
    #[ensures((^self).deep_model() == ((*self).deep_model() & !rhs.deep_model()))]
    fn sub_assign(&mut self, rhs: Self) {
        *self = Self::from_bits_retain(self.bits & !rhs.bits);
    }
}
