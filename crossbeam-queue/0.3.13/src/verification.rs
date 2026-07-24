//! Creusot-facing model of the queues' exclusive-access FIFO state machines.
//!
//! The runtime implementation remains in `array_queue.rs` and `seg_queue.rs`.
//! This module deliberately does not model concurrent interference or claim a
//! linearization proof for the atomic implementations.

use alloc::vec::Vec;
#[allow(unused_imports)]
use creusot_std::prelude::{ensures, logic, pearlite, requires, Int, Invariant, Seq, View};

/// A bounded FIFO queue, represented during verification by its logical values.
#[allow(missing_debug_implementations)]
pub struct ArrayQueue<T> {
    values: Vec<T>,
    cap: usize,
}

impl<T> View for ArrayQueue<T> {
    type ViewTy = Seq<T>;

    #[logic]
    fn view(self) -> Seq<T> {
        pearlite! { self.values@ }
    }
}

impl<T> Invariant for ArrayQueue<T> {
    #[logic(prophetic)]
    fn invariant(self) -> bool {
        pearlite! {
            0 < self.cap@
                && self@.len() <= self.cap@
                && self.cap@ <= usize::MAX@
        }
    }
}

impl<T> ArrayQueue<T> {
    /// Logical capacity used by contracts.
    #[logic]
    pub fn capacity_logic(self) -> Int {
        pearlite! { self.cap@ }
    }

    /// Creates an empty bounded queue.
    #[requires(0 < cap@)]
    #[ensures(result@ == Seq::empty())]
    #[ensures(result.capacity_logic() == cap@)]
    #[ensures(result.invariant())]
    pub fn new(cap: usize) -> Self {
        assert!(cap > 0, "capacity must be non-zero");
        Self {
            values: Vec::new(),
            cap,
        }
    }

    /// Returns the queue capacity.
    #[ensures(result@ == self.capacity_logic())]
    pub fn capacity(&self) -> usize {
        self.cap
    }

    /// Returns the number of queued values.
    #[ensures(result@ == self@.len())]
    pub fn len(&self) -> usize {
        self.values.len()
    }

    /// Returns whether no value is queued.
    #[ensures(result == (self@.len() == 0))]
    pub fn is_empty(&self) -> bool {
        self.values.len() == 0
    }

    /// Returns whether the bounded queue has reached capacity.
    #[requires(self.invariant())]
    #[ensures(result == (self@.len() == self.capacity_logic()))]
    pub fn is_full(&self) -> bool {
        self.values.len() == self.cap
    }

    /// Pushes at the logical tail under exclusive access.
    #[requires(self.invariant())]
    #[ensures(match result {
        Ok(()) => (^self)@ == self@.push_back(value),
        Err(returned) => (^self)@ == self@ && returned == value,
    })]
    #[ensures((^self).capacity_logic() == self.capacity_logic())]
    #[ensures((^self).invariant())]
    pub fn push_mut(&mut self, value: T) -> Result<(), T> {
        if self.values.len() == self.cap {
            Err(value)
        } else {
            self.values.push(value);
            Ok(())
        }
    }

    /// Pops the logical head under exclusive access.
    #[requires(self.invariant())]
    #[ensures(match result {
        Some(value) => self@ == (^self)@.push_front(value),
        None => (^self)@ == self@ && self@.len() == 0,
    })]
    #[ensures((^self).capacity_logic() == self.capacity_logic())]
    #[ensures((^self).invariant())]
    pub fn pop_mut(&mut self) -> Option<T> {
        if self.values.len() == 0 {
            None
        } else {
            Some(self.values.remove(0))
        }
    }
}

/// An unbounded FIFO queue, represented during verification by its values.
#[allow(missing_debug_implementations)]
pub struct SegQueue<T> {
    values: Vec<T>,
}

impl<T> View for SegQueue<T> {
    type ViewTy = Seq<T>;

    #[logic]
    fn view(self) -> Seq<T> {
        pearlite! { self.values@ }
    }
}

impl<T> Invariant for SegQueue<T> {
    #[logic(prophetic)]
    fn invariant(self) -> bool {
        pearlite! { self@.len() <= usize::MAX@ }
    }
}

impl<T> SegQueue<T> {
    /// Creates an empty unbounded queue.
    #[ensures(result@ == Seq::empty())]
    #[ensures(result.invariant())]
    pub const fn new() -> Self {
        Self { values: Vec::new() }
    }

    /// Returns the number of queued values.
    #[ensures(result@ == self@.len())]
    pub fn len(&self) -> usize {
        self.values.len()
    }

    /// Returns whether no value is queued.
    #[ensures(result == (self@.len() == 0))]
    pub fn is_empty(&self) -> bool {
        self.values.len() == 0
    }

    /// Pushes at the logical tail under exclusive access.
    #[requires(self.invariant())]
    #[requires(self@.len() < usize::MAX@)]
    #[ensures((^self)@ == self@.push_back(value))]
    #[ensures((^self).invariant())]
    pub fn push_mut(&mut self, value: T) {
        self.values.push(value);
    }

    /// Pops the logical head under exclusive access.
    #[requires(self.invariant())]
    #[ensures(match result {
        Some(value) => self@ == (^self)@.push_front(value),
        None => (^self)@ == self@ && self@.len() == 0,
    })]
    #[ensures((^self).invariant())]
    pub fn pop_mut(&mut self) -> Option<T> {
        if self.values.len() == 0 {
            None
        } else {
            Some(self.values.remove(0))
        }
    }
}

impl<T> Default for SegQueue<T> {
    #[ensures(result@ == Seq::empty())]
    #[ensures(result.invariant())]
    fn default() -> Self {
        Self::new()
    }
}
