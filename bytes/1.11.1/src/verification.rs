//! Creusot-facing model of the public buffer state machine.
//!
//! The runtime build uses the upstream raw-pointer implementation. This module
//! isolates the logical length and capacity transitions from that representation
//! until Creusot can translate its generic allocation and reference-counting code.

#[allow(unused_imports)]
use creusot_std::prelude::{ensures, logic, pearlite, requires, DeepModel, Int, Invariant, View};

/// An immutable byte buffer, modeled by the length of its visible slice.
pub struct Bytes {
    len: usize,
}

impl View for Bytes {
    type ViewTy = Int;

    #[logic]
    fn view(self) -> Int {
        self.len.deep_model()
    }
}

impl Invariant for Bytes {
    #[logic(prophetic)]
    fn invariant(self) -> bool {
        pearlite! { 0 <= self@ }
    }
}

impl Bytes {
    /// Creates an empty buffer.
    #[ensures(result@ == 0)]
    pub const fn new() -> Self {
        Self { len: 0 }
    }

    /// Returns the visible byte length.
    #[ensures(result@ == self@)]
    pub const fn len(&self) -> usize {
        self.len
    }

    /// Returns true exactly when the visible byte range is empty.
    #[ensures(result == (self@ == 0))]
    pub const fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Splits at `at`, retaining the prefix and returning the suffix.
    #[requires(at@ <= self@)]
    #[ensures((^self)@ == at@)]
    #[ensures(result@ == self@ - at@)]
    pub fn split_off(&mut self, at: usize) -> Self {
        let suffix = Self { len: self.len - at };
        self.len = at;
        suffix
    }

    /// Splits at `at`, returning the prefix and retaining the suffix.
    #[requires(at@ <= self@)]
    #[ensures(result@ == at@)]
    #[ensures((^self)@ == self@ - at@)]
    pub fn split_to(&mut self, at: usize) -> Self {
        let suffix_len = self.len - at;
        self.len = suffix_len;
        Self { len: at }
    }

    /// Shortens the visible range to at most `len` bytes.
    #[ensures((^self)@ == if len@ < self@ { len@ } else { self@ })]
    pub fn truncate(&mut self, len: usize) {
        if len < self.len {
            self.len = len;
        }
    }

    /// Removes every visible byte.
    #[ensures((^self)@ == 0)]
    pub fn clear(&mut self) {
        self.len = 0;
    }
}

/// A mutable byte buffer, modeled by its initialized length and capacity.
pub struct BytesMut {
    len: usize,
    cap: usize,
}

impl View for BytesMut {
    type ViewTy = Int;

    #[logic]
    fn view(self) -> Int {
        self.len.deep_model()
    }
}

impl Invariant for BytesMut {
    #[logic(prophetic)]
    fn invariant(self) -> bool {
        pearlite! { 0 <= self@ && self@ <= self.cap@ }
    }
}

impl BytesMut {
    /// Logical capacity used by public contracts.
    #[logic]
    pub fn capacity_logic(self) -> Int {
        self.cap.deep_model()
    }

    /// Creates an empty mutable buffer without reserved capacity.
    #[ensures(result@ == 0)]
    #[ensures(result.capacity_logic() == 0)]
    pub const fn new() -> Self {
        Self { len: 0, cap: 0 }
    }

    /// Creates an empty mutable buffer with the requested capacity.
    #[ensures(result@ == 0)]
    #[ensures(result.capacity_logic() == capacity@)]
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            len: 0,
            cap: capacity,
        }
    }

    /// Returns the initialized byte length.
    #[ensures(result@ == self@)]
    pub const fn len(&self) -> usize {
        self.len
    }

    /// Returns true exactly when no initialized bytes remain.
    #[ensures(result == (self@ == 0))]
    pub const fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Returns the total capacity of the buffer's current view.
    #[ensures(result@ == self.capacity_logic())]
    pub const fn capacity(&self) -> usize {
        self.cap
    }

    /// Splits at `at`, retaining the prefix and returning the suffix.
    #[requires(at@ <= self.capacity_logic())]
    #[ensures((^self)@ == if self@ < at@ { self@ } else { at@ })]
    #[ensures(result@ == if self@ < at@ { 0 } else { self@ - at@ })]
    #[ensures((^self).capacity_logic() == at@)]
    #[ensures(result.capacity_logic() == self.capacity_logic() - at@)]
    pub fn split_off(&mut self, at: usize) -> Self {
        let result = Self {
            len: if self.len < at { 0 } else { self.len - at },
            cap: self.cap - at,
        };
        if at < self.len {
            self.len = at;
        }
        self.cap = at;
        result
    }

    /// Splits at `at`, returning the prefix and retaining the suffix.
    #[requires(at@ <= self@)]
    #[ensures(result@ == at@)]
    #[ensures((^self)@ == self@ - at@)]
    pub fn split_to(&mut self, at: usize) -> Self {
        let old_len = self.len;
        self.len = old_len - at;
        self.cap = self.cap - at;
        Self { len: at, cap: at }
    }

    /// Shortens the initialized range to at most `len` bytes.
    #[ensures((^self)@ == if len@ < self@ { len@ } else { self@ })]
    #[ensures((^self).capacity_logic() == self.capacity_logic())]
    pub fn truncate(&mut self, len: usize) {
        if len < self.len {
            self.len = len;
        }
    }

    /// Removes every initialized byte without changing capacity.
    #[ensures((^self)@ == 0)]
    #[ensures((^self).capacity_logic() == self.capacity_logic())]
    pub fn clear(&mut self) {
        self.len = 0;
    }

    /// Changes the initialized length, growing capacity when required.
    #[ensures((^self)@ == new_len@)]
    #[ensures((^self).capacity_logic() >= self.capacity_logic())]
    #[ensures((^self).capacity_logic() >= new_len@)]
    pub fn resize(&mut self, new_len: usize, _value: u8) {
        if new_len > self.cap {
            self.cap = new_len;
        }
        self.len = new_len;
    }

    /// Converts into an immutable view with the same visible length.
    #[ensures(result@ == self@)]
    pub fn freeze(self) -> Bytes {
        Bytes { len: self.len }
    }
}

/// Read access to a cursor-based byte buffer.
pub trait Buf: View<ViewTy = Int> {
    /// Returns the number of remaining bytes.
    #[ensures(result@ == self@)]
    fn remaining(&self) -> usize;

    /// Advances the cursor by `cnt` bytes.
    #[requires(cnt@ <= self@)]
    #[ensures((^self)@ == self@ - cnt@)]
    fn advance(&mut self, cnt: usize);
}

/// Write access to a byte buffer.
pub trait BufMut {
    /// Returns writable capacity remaining without reallocating.
    fn remaining_mut(&self) -> usize;
}

impl Buf for Bytes {
    #[ensures(result@ == self@)]
    fn remaining(&self) -> usize {
        self.len
    }

    #[requires(cnt@ <= self@)]
    #[ensures((^self)@ == self@ - cnt@)]
    fn advance(&mut self, cnt: usize) {
        self.len -= cnt;
    }
}

impl Buf for BytesMut {
    #[ensures(result@ == self@)]
    fn remaining(&self) -> usize {
        self.len
    }

    #[requires(cnt@ <= self@)]
    #[ensures((^self)@ == self@ - cnt@)]
    #[ensures((^self).capacity_logic() == self.capacity_logic() - cnt@)]
    fn advance(&mut self, cnt: usize) {
        self.len -= cnt;
        self.cap -= cnt;
    }
}

impl BufMut for BytesMut {
    fn remaining_mut(&self) -> usize {
        // This proof-only trait is intentionally weaker than upstream BufMut,
        // whose remaining_mut reports the maximum allocatable length.
        self.cap - self.len
    }
}
