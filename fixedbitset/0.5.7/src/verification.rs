//! Creusot-facing model of the core `FixedBitSet` state machine.

extern crate alloc;

use alloc::{vec, vec::Vec};
#[allow(unused_imports)]
use creusot_std::prelude::{
    ensures, invariant, logic, pearlite, proof_assert, requires, snapshot, variant, DeepModel, Int,
    Invariant, Seq, View,
};

/// Storage block exposed by the upstream API.
pub type Block = usize;

/// A fixed-length sequence of Boolean bits.
pub struct FixedBitSet {
    bits: Vec<bool>,
}

/// Read one logical bit, extending a finite bitset with disabled bits.
#[logic(open)]
pub fn bit_or_false(bits: Seq<bool>, index: Int) -> bool {
    pearlite! { if 0 <= index && index < bits.len() { bits[index] } else { false } }
}

/// Length of a union-shaped result.
#[logic(open)]
pub fn max_len(left: Int, right: Int) -> Int {
    pearlite! { if left >= right { left } else { right } }
}

/// Select one of union, intersection, difference, or symmetric difference.
#[logic(open)]
pub fn combine_bits(left: bool, right: bool, mode: u8) -> bool {
    pearlite! {
        if mode == 0u8 {
            left || right
        } else if mode == 1u8 {
            left && right
        } else if mode == 2u8 {
            left && !right
        } else {
            left != right
        }
    }
}

/// Count selected bits in the first `count` positions of two zero-extended
/// finite bitsets.
#[logic]
#[requires(0 <= count)]
#[variant(count)]
pub fn binary_count(left: Seq<bool>, right: Seq<bool>, count: Int, mode: u8) -> Int {
    if count == 0 {
        0
    } else {
        pearlite! {
            binary_count(left, right, count - 1, mode)
                + if combine_bits(
                    bit_or_false(left, count - 1),
                    bit_or_false(right, count - 1),
                    mode,
                ) { 1 } else { 0 }
        }
    }
}

#[logic]
#[ensures(binary_count(left, right, 0, mode) == 0)]
fn binary_count_zero(left: Seq<bool>, right: Seq<bool>, mode: u8) {}

#[logic]
#[requires(0 <= count)]
#[ensures(binary_count(left, right, count + 1, mode)
    == binary_count(left, right, count, mode)
        + if combine_bits(bit_or_false(left, count), bit_or_false(right, count), mode) {
            1
        } else {
            0
        })]
fn binary_count_succ(left: Seq<bool>, right: Seq<bool>, count: Int, mode: u8) {}

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

    /// Return whether every bit is disabled.
    #[ensures(result == (forall<i> 0 <= i && i < self@.len() ==> self@[i] == false))]
    pub fn is_clear(&self) -> bool {
        let mut index = 0usize;
        #[invariant(index@ <= self@.len())]
        #[invariant(forall<i> 0 <= i && i < index@ ==> self@[i] == false)]
        #[variant(self@.len() - index@)]
        while index < self.bits.len() {
            if self.bits[index] {
                return false;
            }
            index += 1;
        }
        true
    }

    /// Return whether every bit is enabled.
    #[ensures(result == (forall<i> 0 <= i && i < self@.len() ==> self@[i] == true))]
    pub fn is_full(&self) -> bool {
        let mut index = 0usize;
        #[invariant(index@ <= self@.len())]
        #[invariant(forall<i> 0 <= i && i < index@ ==> self@[i] == true)]
        #[variant(self@.len() - index@)]
        while index < self.bits.len() {
            if !self.bits[index] {
                return false;
            }
            index += 1;
        }
        true
    }

    /// Find the least enabled bit.
    #[ensures(match result {
        None => forall<i> 0 <= i && i < self@.len() ==> self@[i] == false,
        Some(index) => index@ < self@.len()
            && self@[index@] == true
            && (forall<i> 0 <= i && i < index@ ==> self@[i] == false),
    })]
    pub fn minimum(&self) -> Option<usize> {
        let mut index = 0usize;
        #[invariant(index@ <= self@.len())]
        #[invariant(forall<i> 0 <= i && i < index@ ==> self@[i] == false)]
        #[variant(self@.len() - index@)]
        while index < self.bits.len() {
            if self.bits[index] {
                return Some(index);
            }
            index += 1;
        }
        None
    }

    /// Find the greatest enabled bit.
    #[ensures(match result {
        None => forall<i> 0 <= i && i < self@.len() ==> self@[i] == false,
        Some(index) => index@ < self@.len()
            && self@[index@] == true
            && (forall<i> index@ < i && i < self@.len() ==> self@[i] == false),
    })]
    pub fn maximum(&self) -> Option<usize> {
        let mut index = self.bits.len();
        #[invariant(index@ <= self@.len())]
        #[invariant(forall<i> index@ <= i && i < self@.len() ==> self@[i] == false)]
        #[variant(index@)]
        while index > 0 {
            index -= 1;
            if self.bits[index] {
                return Some(index);
            }
        }
        None
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

    /// Return whether the two finite sets have no enabled index in common.
    #[ensures(result == (forall<i> 0 <= i && i < self@.len() ==>
        !(self@[i] && (if i < other@.len() { other@[i] } else { false }))))]
    pub fn is_disjoint(&self, other: &FixedBitSet) -> bool {
        let mut index = 0usize;
        #[invariant(index@ <= self@.len())]
        #[invariant(forall<i> 0 <= i && i < index@ ==>
            !(self@[i] && (if i < other@.len() { other@[i] } else { false })))]
        #[variant(self@.len() - index@)]
        while index < self.bits.len() {
            if self.bits[index] && other.contains(index) {
                return false;
            }
            index += 1;
        }
        true
    }

    /// Return whether every enabled bit in `self` is enabled in `other`.
    #[ensures(result == (forall<i> 0 <= i && i < self@.len() ==>
        self@[i] ==> if i < other@.len() { other@[i] } else { false }))]
    pub fn is_subset(&self, other: &FixedBitSet) -> bool {
        let mut index = 0usize;
        #[invariant(index@ <= self@.len())]
        #[invariant(forall<i> 0 <= i && i < index@ ==>
            self@[i] ==> if i < other@.len() { other@[i] } else { false })]
        #[variant(self@.len() - index@)]
        while index < self.bits.len() {
            if self.bits[index] && !other.contains(index) {
                return false;
            }
            index += 1;
        }
        true
    }

    /// Return whether every enabled bit in `other` is enabled in `self`.
    #[ensures(result == (forall<i> 0 <= i && i < other@.len() ==>
        other@[i] ==> if i < self@.len() { self@[i] } else { false }))]
    pub fn is_superset(&self, other: &FixedBitSet) -> bool {
        other.is_subset(self)
    }

    /// Replace `self` by the union, growing it to the longer finite length.
    #[ensures((^self)@.len() == max_len(self@.len(), other@.len()))]
    #[ensures(forall<i> 0 <= i && i < (^self)@.len() ==>
        (^self)@[i] == (bit_or_false(self@, i) || bit_or_false(other@, i)))]
    pub fn union_with(&mut self, other: &FixedBitSet) {
        let old = snapshot!(self@);
        let rhs = snapshot!(other@);
        self.grow(other.bits.len());
        let mut index = 0usize;
        #[invariant(self@.len() == max_len(old.len(), rhs.len()))]
        #[invariant(index@ <= rhs.len())]
        #[invariant(forall<i> 0 <= i && i < self@.len() ==>
            self@[i] == if i < index@ {
                bit_or_false(*old, i) || bit_or_false(*rhs, i)
            } else {
                bit_or_false(*old, i)
            })]
        #[variant(rhs.len() - index@)]
        while index < other.bits.len() {
            let enabled = self.bits[index] || other.bits[index];
            self.bits[index] = enabled;
            index += 1;
        }
    }

    /// Replace `self` by the intersection without changing its finite length.
    #[ensures((^self)@.len() == self@.len())]
    #[ensures(forall<i> 0 <= i && i < self@.len() ==>
        (^self)@[i] == (self@[i] && bit_or_false(other@, i)))]
    pub fn intersect_with(&mut self, other: &FixedBitSet) {
        let old = snapshot!(self@);
        let rhs = snapshot!(other@);
        let mut index = 0usize;
        #[invariant(self@.len() == old.len())]
        #[invariant(index@ <= old.len())]
        #[invariant(forall<i> 0 <= i && i < self@.len() ==>
            self@[i] == if i < index@ {
                old[i] && bit_or_false(*rhs, i)
            } else {
                old[i]
            })]
        #[variant(old.len() - index@)]
        while index < self.bits.len() {
            let enabled = self.bits[index] && other.contains(index);
            self.bits[index] = enabled;
            index += 1;
        }
    }

    /// Remove every bit present in `other` without changing the finite length.
    #[ensures((^self)@.len() == self@.len())]
    #[ensures(forall<i> 0 <= i && i < self@.len() ==>
        (^self)@[i] == (self@[i] && !bit_or_false(other@, i)))]
    pub fn difference_with(&mut self, other: &FixedBitSet) {
        let old = snapshot!(self@);
        let rhs = snapshot!(other@);
        let mut index = 0usize;
        #[invariant(self@.len() == old.len())]
        #[invariant(index@ <= old.len())]
        #[invariant(forall<i> 0 <= i && i < self@.len() ==>
            self@[i] == if i < index@ {
                old[i] && !bit_or_false(*rhs, i)
            } else {
                old[i]
            })]
        #[variant(old.len() - index@)]
        while index < self.bits.len() {
            let enabled = self.bits[index] && !other.contains(index);
            self.bits[index] = enabled;
            index += 1;
        }
    }

    /// Replace `self` by the symmetric difference, growing to the longer
    /// finite length.
    #[ensures((^self)@.len() == max_len(self@.len(), other@.len()))]
    #[ensures(forall<i> 0 <= i && i < (^self)@.len() ==>
        (^self)@[i] == (bit_or_false(self@, i) != bit_or_false(other@, i)))]
    pub fn symmetric_difference_with(&mut self, other: &FixedBitSet) {
        let old = snapshot!(self@);
        let rhs = snapshot!(other@);
        self.grow(other.bits.len());
        let mut index = 0usize;
        #[invariant(self@.len() == max_len(old.len(), rhs.len()))]
        #[invariant(index@ <= rhs.len())]
        #[invariant(forall<i> 0 <= i && i < self@.len() ==>
            self@[i] == if i < index@ {
                bit_or_false(*old, i) != bit_or_false(*rhs, i)
            } else {
                bit_or_false(*old, i)
            })]
        #[variant(rhs.len() - index@)]
        while index < other.bits.len() {
            let enabled = self.bits[index] != other.bits[index];
            self.bits[index] = enabled;
            index += 1;
        }
    }

    #[requires(mode@ <= 3)]
    #[ensures(result@ == binary_count(
        self@,
        other@,
        max_len(self@.len(), other@.len()),
        mode,
    ))]
    fn count_binary(&self, other: &FixedBitSet, mode: u8) -> usize {
        let length = if self.bits.len() >= other.bits.len() {
            self.bits.len()
        } else {
            other.bits.len()
        };
        let mut index = 0usize;
        let mut count = 0usize;
        proof_assert! {
            binary_count_zero(self@, other@, mode);
            binary_count(self@, other@, 0, mode) == 0
        };
        #[invariant(index@ <= length@)]
        #[invariant(length@ == max_len(self@.len(), other@.len()))]
        #[invariant(count@ == binary_count(self@, other@, index@, mode))]
        #[invariant(count@ <= index@)]
        #[variant(length@ - index@)]
        while index < length {
            let left = self.contains(index);
            let right = other.contains(index);
            let selected = if mode == 0 {
                left || right
            } else if mode == 1 {
                left && right
            } else if mode == 2 {
                left && !right
            } else {
                left != right
            };
            proof_assert! {
                binary_count_succ(self@, other@, index@, mode);
                binary_count(self@, other@, index@ + 1, mode)
                    == binary_count(self@, other@, index@, mode)
                        + if combine_bits(
                            bit_or_false(self@, index@),
                            bit_or_false(other@, index@),
                            mode,
                        ) { 1 } else { 0 }
            };
            if selected {
                count += 1;
            }
            index += 1;
        }
        count
    }

    /// Count enabled bits in the union without mutating either operand.
    #[ensures(result@ == binary_count(
        self@,
        other@,
        max_len(self@.len(), other@.len()),
        0u8,
    ))]
    pub fn union_count(&self, other: &FixedBitSet) -> usize {
        self.count_binary(other, 0)
    }

    /// Count enabled bits in the intersection without mutation.
    #[ensures(result@ == binary_count(
        self@,
        other@,
        max_len(self@.len(), other@.len()),
        1u8,
    ))]
    pub fn intersection_count(&self, other: &FixedBitSet) -> usize {
        self.count_binary(other, 1)
    }

    /// Count bits enabled in `self` but not `other` without mutation.
    #[ensures(result@ == binary_count(
        self@,
        other@,
        max_len(self@.len(), other@.len()),
        2u8,
    ))]
    pub fn difference_count(&self, other: &FixedBitSet) -> usize {
        self.count_binary(other, 2)
    }

    /// Count enabled bits in the symmetric difference without mutation.
    #[ensures(result@ == binary_count(
        self@,
        other@,
        max_len(self@.len(), other@.len()),
        3u8,
    ))]
    pub fn symmetric_difference_count(&self, other: &FixedBitSet) -> usize {
        self.count_binary(other, 3)
    }
}

impl Default for FixedBitSet {
    #[ensures(result@ == Seq::empty())]
    fn default() -> Self {
        Self::new()
    }
}
