//! Creusot-facing model of the core `FixedBitSet` state machine.

extern crate alloc;

use alloc::{vec, vec::Vec};
#[allow(unused_imports)]
use creusot_std::prelude::{
    ensures, invariant, logic, pearlite, requires, snapshot, variant, DeepModel, Int, Invariant,
    Seq, View,
};

/// Storage block exposed by the upstream API.
pub type Block = usize;

/// A fixed-length sequence of Boolean bits.
pub struct FixedBitSet {
    bits: Vec<bool>,
}

impl View for FixedBitSet {
    type ViewTy = Seq<bool>;

    #[logic]
    fn view(self) -> Seq<bool> {
        pearlite! { self.bits@ }
    }
}

impl Invariant for FixedBitSet {
    #[logic(prophetic)]
    fn invariant(self) -> bool {
        pearlite! { 0 <= self@.len() && self@.len() <= usize::MAX@ }
    }
}

impl FixedBitSet {
    /// Create a bitset with no bits.
    #[ensures(result@ == Seq::empty())]
    pub const fn new() -> Self {
        Self { bits: Vec::new() }
    }

    /// Create `bits` disabled bits.
    #[ensures(result@.len() == bits@)]
    #[ensures(forall<i> 0 <= i && i < bits@ ==> result@[i] == false)]
    pub fn with_capacity(bits: usize) -> Self {
        Self {
            bits: vec![false; bits],
        }
    }

    /// Return the fixed length in bits.
    #[ensures(result@ == self@.len())]
    pub fn len(&self) -> usize {
        self.bits.len()
    }

    /// Return whether the bitset has length zero.
    #[ensures(result == (self@.len() == 0))]
    pub fn is_empty(&self) -> bool {
        self.bits.len() == 0
    }

    /// Return a bit, treating indices beyond the fixed length as disabled.
    #[ensures(result == if bit@ < self@.len() { self@[bit@] } else { false })]
    pub fn contains(&self, bit: usize) -> bool {
        if bit < self.bits.len() {
            self.bits[bit]
        } else {
            false
        }
    }

    /// Grow to `bits` bits if necessary, preserving the old prefix and clearing
    /// every newly allocated bit.
    #[ensures((^self)@.len() == if bits@ > self@.len() { bits@ } else { self@.len() })]
    #[ensures(forall<i> 0 <= i && i < self@.len() ==> (^self)@[i] == self@[i])]
    #[ensures(forall<i> self@.len() <= i && i < (^self)@.len() ==> (^self)@[i] == false)]
    pub fn grow(&mut self, bits: usize) {
        if self.bits.len() < bits {
            let old = snapshot!(self@);
            #[invariant(old.len() <= self@.len() && self@.len() <= bits@)]
            #[invariant(forall<i> 0 <= i && i < old.len() ==> self@[i] == old[i])]
            #[invariant(forall<i> old.len() <= i && i < self@.len() ==> self@[i] == false)]
            #[variant(bits@ - self@.len())]
            while self.bits.len() < bits {
                self.bits.push(false);
            }
        }
    }

    /// Clear every bit without changing the fixed length.
    #[ensures((^self)@.len() == self@.len())]
    #[ensures(forall<i> 0 <= i && i < self@.len() ==> (^self)@[i] == false)]
    pub fn clear(&mut self) {
        self.bits = vec![false; self.bits.len()];
    }

    /// Enable one in-range bit.
    #[requires(bit@ < self@.len())]
    #[ensures((^self)@ == self@.set(bit@, true))]
    pub fn insert(&mut self, bit: usize) {
        self.bits[bit] = true;
    }

    /// Disable one in-range bit.
    #[requires(bit@ < self@.len())]
    #[ensures((^self)@ == self@.set(bit@, false))]
    pub fn remove(&mut self, bit: usize) {
        self.bits[bit] = false;
    }

    /// Enable one bit and return its previous value.
    #[requires(bit@ < self@.len())]
    #[ensures(result == self@[bit@])]
    #[ensures((^self)@ == self@.set(bit@, true))]
    pub fn put(&mut self, bit: usize) -> bool {
        let previous = self.bits[bit];
        self.bits[bit] = true;
        previous
    }

    /// Invert one in-range bit.
    #[requires(bit@ < self@.len())]
    #[ensures((^self)@ == self@.set(bit@, !self@[bit@]))]
    pub fn toggle(&mut self, bit: usize) {
        let enabled = self.bits[bit];
        self.bits[bit] = !enabled;
    }

    /// Set one in-range bit to the requested state.
    #[requires(bit@ < self@.len())]
    #[ensures((^self)@ == self@.set(bit@, enabled))]
    pub fn set(&mut self, bit: usize, enabled: bool) {
        self.bits[bit] = enabled;
    }

    /// Copy `from` to `to`; an out-of-range source is treated as disabled.
    #[requires(to@ < self@.len())]
    #[ensures((^self)@ == self@.set(to@,
        if from@ < self@.len() { self@[from@] } else { false }))]
    pub fn copy_bit(&mut self, from: usize, to: usize) {
        let enabled = self.contains(from);
        self.set(to, enabled);
    }

    /// Grow as needed and enable `bit`.
    #[requires(bit@ < usize::MAX@)]
    #[ensures((^self)@.len() == if bit@ + 1 > self@.len() { bit@ + 1 } else { self@.len() })]
    #[ensures((^self)@[bit@] == true)]
    #[ensures(forall<i> 0 <= i && i < self@.len() && i != bit@ ==> (^self)@[i] == self@[i])]
    #[ensures(forall<i> self@.len() <= i && i < (^self)@.len() && i != bit@ ==> (^self)@[i] == false)]
    pub fn grow_and_insert(&mut self, bit: usize) {
        self.grow(bit + 1);
        self.insert(bit);
    }
}

impl Default for FixedBitSet {
    #[ensures(result@ == Seq::empty())]
    fn default() -> Self {
        Self::new()
    }
}
