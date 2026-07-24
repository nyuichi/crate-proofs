//! Adler-32 checksum implementation.
//!
//! This implementation features:
//!
//! - Permissively licensed (0BSD) clean-room implementation.
//! - Zero dependencies.
//! - Zero `unsafe`.
//! - Decent performance (3-4 GB/s).
//! - `#![no_std]` support (with `default-features = false`).

#![doc(html_root_url = "https://docs.rs/adler2/2.0.0")]
// Deny a few warnings in doctests, since rustdoc `allow`s many warnings by default
#![doc(test(attr(deny(unused_imports, unused_must_use))))]
#![cfg_attr(docsrs, feature(doc_cfg))]
#![warn(missing_debug_implementations)]
#![allow(unexpected_cfgs)]
#![cfg_attr(creusot, allow(missing_debug_implementations))]
#![forbid(unsafe_code)]
#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(not(feature = "std"))]
extern crate core as std;

extern crate creusot_std;

#[allow(unused_imports)]
use creusot_std::prelude::{
    ensures, invariant, logic, pearlite, proof_assert, requires, trusted, variant, DeepModel, Int,
    Invariant, Seq, View,
};

mod algo;

use std::hash::Hasher;

#[cfg(feature = "std")]
use std::io::{self, BufRead};

/// Adler-32 checksum calculator.
///
/// An instance of this type is equivalent to an Adler-32 checksum: It can be created in the default
/// state via [`new`] (or the provided `Default` impl), or from a precalculated checksum via
/// [`from_checksum`], and the currently stored checksum can be fetched via [`checksum`].
///
/// This type also implements `Hasher`, which makes it easy to calculate Adler-32 checksums of any
/// type that implements or derives `Hash`. This also allows using Adler-32 in a `HashMap`, although
/// that is not recommended (while every checksum is a hash function, they are not necessarily a
/// good one).
///
/// # Examples
///
/// Basic, piecewise checksum calculation:
///
/// ```
/// use adler2::Adler32;
///
/// let mut adler = Adler32::new();
///
/// adler.write_slice(&[0, 1, 2]);
/// adler.write_slice(&[3, 4, 5]);
///
/// assert_eq!(adler.checksum(), 0x00290010);
/// ```
///
/// Using `Hash` to process structures:
///
/// ```
/// use std::hash::Hash;
/// use adler2::Adler32;
///
/// #[derive(Hash)]
/// struct Data {
///     byte: u8,
///     word: u16,
///     big: u64,
/// }
///
/// let mut adler = Adler32::new();
///
/// let data = Data { byte: 0x1F, word: 0xABCD, big: !0 };
/// data.hash(&mut adler);
///
/// // hash value depends on architecture endianness
/// if cfg!(target_endian = "little") {
///     assert_eq!(adler.checksum(), 0x33410990);
/// }
/// if cfg!(target_endian = "big") {
///     assert_eq!(adler.checksum(), 0x331F0990);
/// }
///
/// ```
///
/// [`new`]: #method.new
/// [`from_checksum`]: #method.from_checksum
/// [`checksum`]: #method.checksum
#[derive(Copy, Clone)]
#[cfg_attr(not(creusot), derive(Debug))]
pub struct Adler32 {
    a: u16,
    b: u16,
}

impl View for Adler32 {
    type ViewTy = (u16, u16);

    /// The two reduced Adler-32 accumulators `(a, b)`.
    #[logic]
    fn view(self) -> Self::ViewTy {
        (self.a, self.b)
    }
}

impl DeepModel for Adler32 {
    type DeepModelTy = (Int, Int);

    /// Mathematical integer form of the two Adler-32 accumulators.
    #[logic]
    fn deep_model(self) -> Self::DeepModelTy {
        (self.a.deep_model(), self.b.deep_model())
    }
}

impl Invariant for Adler32 {
    /// Every pair of stored `u16` accumulators is a valid representation.
    ///
    /// `from_checksum` deliberately accepts every `u32`, including values whose
    /// halves are not canonical residues modulo 65521. The next update reduces
    /// both accumulators to canonical residues.
    #[logic(open)]
    fn invariant(self) -> bool {
        pearlite! { true }
    }
}

/// One mathematical Adler-32 transition.
#[logic(open)]
pub fn adler32_step(state: (Int, Int), byte: u8) -> (Int, Int) {
    pearlite! {
        let next_a = (state.0 + byte@) % 65521;
        (next_a, (state.1 + next_a) % 65521)
    }
}

/// Sum of all byte values in `bytes`.
#[logic]
#[variant(bytes.len())]
#[ensures(bytes.len() == 0 ==> result == 0)]
#[ensures(bytes.len() > 0 ==> result == adler32_byte_sum(
    bytes.subsequence(0, bytes.len() - 1)
) + bytes[bytes.len() - 1]@)]
pub fn adler32_byte_sum(bytes: Seq<u8>) -> Int {
    if bytes.len() == 0 {
        0
    } else {
        adler32_byte_sum(bytes.subsequence(0, bytes.len() - 1))
            + bytes[bytes.len() - 1].deep_model()
    }
}

/// Weighted byte sum used by Adler-32's second accumulator.
///
/// For `[D1, ..., Dn]` this is `n*D1 + ... + 1*Dn`.
#[logic]
#[variant(bytes.len())]
#[ensures(bytes.len() == 0 ==> result == 0)]
#[ensures(bytes.len() > 0 ==> result == adler32_weighted_sum(
    bytes.subsequence(0, bytes.len() - 1)
) + adler32_byte_sum(bytes))]
pub fn adler32_weighted_sum(bytes: Seq<u8>) -> Int {
    if bytes.len() == 0 {
        0
    } else {
        adler32_weighted_sum(bytes.subsequence(0, bytes.len() - 1)) + adler32_byte_sum(bytes)
    }
}

#[logic]
#[variant(bytes.len())]
#[ensures(0 <= adler32_byte_sum(bytes))]
fn adler32_byte_sum_nonnegative(bytes: Seq<u8>) {
    if bytes.len() > 0 {
        adler32_byte_sum_nonnegative(bytes.subsequence(0, bytes.len() - 1));
    }
}

#[logic]
#[variant(bytes.len())]
#[ensures(0 <= adler32_weighted_sum(bytes))]
fn adler32_weighted_sum_nonnegative(bytes: Seq<u8>) {
    if bytes.len() > 0 {
        let prefix = bytes.subsequence(0, bytes.len() - 1);
        adler32_weighted_sum_nonnegative(prefix);
        adler32_byte_sum_nonnegative(bytes);
    }
}

/// The mathematical Adler-32 state after processing `bytes` from `state`.
#[logic(open)]
pub fn adler32_update(state: (Int, Int), bytes: Seq<u8>) -> (Int, Int) {
    pearlite! {
        (
            (state.0 + adler32_byte_sum(bytes)) % 65521,
            (state.1 + bytes.len() * state.0 + adler32_weighted_sum(bytes)) % 65521,
        )
    }
}

#[logic]
#[ensures(adler32_byte_sum(bytes.push_back(byte))
    == adler32_byte_sum(bytes) + byte@)]
fn adler32_byte_sum_push(bytes: Seq<u8>, byte: u8) {
    proof_assert!(bytes.push_back(byte).len() == bytes.len() + 1);
    proof_assert!(bytes.push_back(byte).subsequence(0, bytes.len()) == bytes);
    proof_assert!(bytes.push_back(byte)[bytes.len()] == byte);
}

#[logic]
#[ensures(adler32_weighted_sum(bytes.push_back(byte))
    == adler32_weighted_sum(bytes) + adler32_byte_sum(bytes) + byte@)]
fn adler32_weighted_sum_push(bytes: Seq<u8>, byte: u8) {
    adler32_byte_sum_push(bytes, byte);
    proof_assert!(bytes.push_back(byte).len() == bytes.len() + 1);
    proof_assert!(bytes.push_back(byte).subsequence(0, bytes.len()) == bytes);
}

#[logic]
#[variant(right.len())]
#[allow(dead_code)]
#[ensures(adler32_byte_sum(left.concat(right))
    == adler32_byte_sum(left) + adler32_byte_sum(right))]
pub(crate) fn adler32_byte_sum_concat(left: Seq<u8>, right: Seq<u8>) {
    if right.len() > 0 {
        let prefix = right.subsequence(0, right.len() - 1);
        let byte = right[right.len() - 1];
        proof_assert!(prefix.len() == right.len() - 1);
        adler32_byte_sum_concat(left, prefix);
        proof_assert!(right.ext_eq(prefix.push_back(byte)));
        proof_assert!(right == prefix.push_back(byte));
        proof_assert!(left
            .concat(right)
            .ext_eq(left.concat(prefix).push_back(byte)));
        proof_assert!(left.concat(right) == left.concat(prefix).push_back(byte));
        adler32_byte_sum_push(left.concat(prefix), byte);
        adler32_byte_sum_push(prefix, byte);
    } else {
        proof_assert!(right.len() == 0);
        proof_assert!(adler32_byte_sum(right) == 0);
        proof_assert!(left.concat(right) == left);
    }
}

#[logic]
#[variant(right.len())]
#[allow(dead_code)]
#[ensures(adler32_weighted_sum(left.concat(right))
    == adler32_weighted_sum(left)
        + right.len() * adler32_byte_sum(left)
        + adler32_weighted_sum(right))]
pub(crate) fn adler32_weighted_sum_concat(left: Seq<u8>, right: Seq<u8>) {
    if right.len() > 0 {
        let prefix = right.subsequence(0, right.len() - 1);
        let byte = right[right.len() - 1];
        proof_assert!(prefix.len() == right.len() - 1);
        adler32_byte_sum_concat(left, right);
        adler32_byte_sum_concat(left, prefix);
        adler32_weighted_sum_concat(left, prefix);
        proof_assert!(right.len() == prefix.len() + 1);
        proof_assert!(right.ext_eq(prefix.push_back(byte)));
        proof_assert!(right == prefix.push_back(byte));
        proof_assert!(left
            .concat(right)
            .ext_eq(left.concat(prefix).push_back(byte)));
        proof_assert!(left.concat(right) == left.concat(prefix).push_back(byte));
        adler32_weighted_sum_push(left.concat(prefix), byte);
        adler32_weighted_sum_push(prefix, byte);
        proof_assert!(
            right.len() * adler32_byte_sum(left)
                == prefix.len() * adler32_byte_sum(left) + adler32_byte_sum(left)
        );
        proof_assert!(
            adler32_weighted_sum(left.concat(right))
                == adler32_weighted_sum(left)
                    + right.len() * adler32_byte_sum(left)
                    + adler32_weighted_sum(right)
        );
    } else {
        proof_assert!(right.len() == 0);
        proof_assert!(adler32_byte_sum(right) == 0);
        proof_assert!(adler32_weighted_sum(right) == 0);
        proof_assert!(left.concat(right) == left);
    }
}

#[logic]
#[requires(bytes.len() >= 4)]
#[ensures(adler32_byte_sum(bytes) == adler32_byte_sum(
    bytes.subsequence(0, bytes.len() - 4)
) + bytes[bytes.len() - 4]@ + bytes[bytes.len() - 3]@
    + bytes[bytes.len() - 2]@ + bytes[bytes.len() - 1]@)]
pub(crate) fn adler32_byte_sum_last_four(bytes: Seq<u8>) {
    let p1 = bytes.subsequence(0, bytes.len() - 1);
    let p2 = p1.subsequence(0, p1.len() - 1);
    let p3 = p2.subsequence(0, p2.len() - 1);
    proof_assert!(p3.subsequence(0, p3.len() - 1) == bytes.subsequence(0, bytes.len() - 4));
    proof_assert!(p3[p3.len() - 1] == bytes[bytes.len() - 4]);
    proof_assert!(p2[p2.len() - 1] == bytes[bytes.len() - 3]);
    proof_assert!(p1[p1.len() - 1] == bytes[bytes.len() - 2]);
}

#[logic]
#[requires(bytes.len() >= 4)]
#[ensures(adler32_weighted_sum(bytes) == adler32_weighted_sum(
    bytes.subsequence(0, bytes.len() - 4)
) + 4 * adler32_byte_sum(bytes.subsequence(0, bytes.len() - 4))
    + 4 * bytes[bytes.len() - 4]@ + 3 * bytes[bytes.len() - 3]@
    + 2 * bytes[bytes.len() - 2]@ + bytes[bytes.len() - 1]@)]
pub(crate) fn adler32_weighted_sum_last_four(bytes: Seq<u8>) {
    let p1 = bytes.subsequence(0, bytes.len() - 1);
    let p2 = p1.subsequence(0, p1.len() - 1);
    let p3 = p2.subsequence(0, p2.len() - 1);
    adler32_byte_sum_last_four(bytes);
    proof_assert!(p3.subsequence(0, p3.len() - 1) == bytes.subsequence(0, bytes.len() - 4));
    proof_assert!(p3[p3.len() - 1] == bytes[bytes.len() - 4]);
    proof_assert!(p2[p2.len() - 1] == bytes[bytes.len() - 3]);
    proof_assert!(p1[p1.len() - 1] == bytes[bytes.len() - 2]);
}

/// Updating any nonnegative state produces a reduced state.
#[logic]
#[requires(0 <= state.0)]
#[requires(0 <= state.1)]
#[ensures(0 <= adler32_update(state, bytes).0
    && adler32_update(state, bytes).0 < 65521)]
#[ensures(0 <= adler32_update(state, bytes).1
    && adler32_update(state, bytes).1 < 65521)]
pub fn adler32_update_reduced(state: (Int, Int), bytes: Seq<u8>) {
    adler32_byte_sum_nonnegative(bytes);
    adler32_weighted_sum_nonnegative(bytes);
}

/// Packs a mathematical Adler-32 state into its public checksum value.
#[logic(open)]
pub fn adler32_pack(state: (Int, Int)) -> Int {
    pearlite! { state.1 * 65536 + state.0 }
}

impl Adler32 {
    /// Creates a new Adler-32 instance with default state.
    #[inline]
    #[ensures(result.deep_model() == (1, 0))]
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates an `Adler32` instance from a precomputed Adler-32 checksum.
    ///
    /// This allows resuming checksum calculation without having to keep the `Adler32` instance
    /// around.
    ///
    /// # Example
    ///
    /// ```
    /// # use adler2::Adler32;
    /// let parts = [
    ///     "rust",
    ///     "acean",
    /// ];
    /// let whole = adler2::adler32_slice(b"rustacean");
    ///
    /// let mut sum = Adler32::new();
    /// sum.write_slice(parts[0].as_bytes());
    /// let partial = sum.checksum();
    ///
    /// // ...later
    ///
    /// let mut sum = Adler32::from_checksum(partial);
    /// sum.write_slice(parts[1].as_bytes());
    /// assert_eq!(sum.checksum(), whole);
    /// ```
    #[inline]
    #[ensures(result.deep_model().0 == sum@ % 65536)]
    #[ensures(result.deep_model().1 == sum@ / 65536)]
    pub const fn from_checksum(sum: u32) -> Self {
        Adler32 {
            a: (sum % 65536) as u16,
            b: (sum / 65536) as u16,
        }
    }

    /// Returns the calculated checksum at this point in time.
    #[inline]
    #[ensures(result@ == adler32_pack(self.deep_model()))]
    pub fn checksum(&self) -> u32 {
        u32::from(self.b) * 65536 + u32::from(self.a)
    }

    /// Adds `bytes` to the checksum calculation.
    ///
    /// If efficiency matters, this should be called with Byte slices that contain at least a few
    /// thousand Bytes.
    #[ensures((^self).deep_model() == adler32_update((*self).deep_model(), bytes@))]
    pub fn write_slice(&mut self, bytes: &[u8]) {
        self.compute(bytes);
    }
}

impl Default for Adler32 {
    #[inline]
    #[ensures(result.deep_model() == (1, 0))]
    fn default() -> Self {
        Adler32 { a: 1, b: 0 }
    }
}

impl Hasher for Adler32 {
    #[inline]
    #[ensures(result@ == adler32_pack(self.deep_model()))]
    fn finish(&self) -> u64 {
        u64::from(self.checksum())
    }

    #[ensures((^self).deep_model() == adler32_update((*self).deep_model(), bytes@))]
    fn write(&mut self, bytes: &[u8]) {
        self.write_slice(bytes);
    }
}

/// Calculates the Adler-32 checksum of a byte slice.
///
/// This is a convenience function around the [`Adler32`] type.
///
/// [`Adler32`]: struct.Adler32.html
#[ensures(result@ == adler32_pack(adler32_update((1, 0), data@)))]
pub fn adler32_slice(data: &[u8]) -> u32 {
    let mut h = Adler32::new();
    h.write_slice(data);
    h.checksum()
}

/// Calculates the Adler-32 checksum of a `BufRead`'s contents.
///
/// The passed `BufRead` implementor will be read until it reaches EOF (or until it reports an
/// error).
///
/// If you only have a `Read` implementor, you can wrap it in `std::io::BufReader` before calling
/// this function.
///
/// # Errors
///
/// Any error returned by the reader are bubbled up by this function.
///
/// # Examples
///
/// ```no_run
/// # fn run() -> Result<(), Box<dyn std::error::Error>> {
/// use adler2::adler32;
///
/// use std::fs::File;
/// use std::io::BufReader;
///
/// let file = File::open("input.txt")?;
/// let mut file = BufReader::new(file);
///
/// adler32(&mut file)?;
/// # Ok(()) }
/// # fn main() { run().unwrap() }
/// ```
#[cfg(feature = "std")]
#[cfg_attr(docsrs, doc(cfg(feature = "std")))]
// Creusot does not currently specify BufRead's stateful fill_buf/consume protocol.
// The checksum core used by this adapter is verified separately.
// TODO: Remove this boundary once BufRead exposes a logical unread-byte sequence
// and contracts connecting fill_buf/consume to that sequence.
#[trusted]
pub fn adler32<R: BufRead>(mut reader: R) -> io::Result<u32> {
    let mut h = Adler32::new();
    loop {
        let len = {
            let buf = reader.fill_buf()?;
            if buf.is_empty() {
                return Ok(h.checksum());
            }

            h.write_slice(buf);
            buf.len()
        };
        reader.consume(len);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn zeroes() {
        assert_eq!(adler32_slice(&[]), 1);
        assert_eq!(adler32_slice(&[0]), 1 | 1 << 16);
        assert_eq!(adler32_slice(&[0, 0]), 1 | 2 << 16);
        assert_eq!(adler32_slice(&[0; 100]), 0x00640001);
        assert_eq!(adler32_slice(&[0; 1024]), 0x04000001);
        assert_eq!(adler32_slice(&[0; 1024 * 1024]), 0x00f00001);
    }

    #[test]
    fn ones() {
        assert_eq!(adler32_slice(&[0xff; 1024]), 0x79a6fc2e);
        assert_eq!(adler32_slice(&[0xff; 1024 * 1024]), 0x8e88ef11);
    }

    #[test]
    fn mixed() {
        assert_eq!(adler32_slice(&[1]), 2 | 2 << 16);
        assert_eq!(adler32_slice(&[40]), 41 | 41 << 16);

        assert_eq!(adler32_slice(&[0xA5; 1024 * 1024]), 0xd5009ab1);
    }

    /// Example calculation from https://en.wikipedia.org/wiki/Adler-32.
    #[test]
    fn wiki() {
        assert_eq!(adler32_slice(b"Wikipedia"), 0x11E60398);
    }

    #[test]
    fn resume() {
        let mut adler = Adler32::new();
        adler.write_slice(&[0xff; 1024]);
        let partial = adler.checksum();
        assert_eq!(partial, 0x79a6fc2e); // from above
        adler.write_slice(&[0xff; 1024 * 1024 - 1024]);
        assert_eq!(adler.checksum(), 0x8e88ef11); // from above

        // Make sure that we can resume computing from the partial checksum via `from_checksum`.
        let mut adler = Adler32::from_checksum(partial);
        adler.write_slice(&[0xff; 1024 * 1024 - 1024]);
        assert_eq!(adler.checksum(), 0x8e88ef11); // from above
    }

    #[test]
    fn resume_from_noncanonical_checksum() {
        let mut sum = Adler32::from_checksum(u32::MAX);
        assert_eq!(sum.checksum(), u32::MAX);

        sum.write_slice(&[0]);
        assert_eq!(sum.checksum(), 0x001C_000E);
    }

    #[cfg(feature = "std")]
    #[test]
    fn bufread() {
        use std::io::BufReader;
        fn test(data: &[u8], checksum: u32) {
            // `BufReader` uses an 8 KB buffer, so this will test buffer refilling.
            let mut buf = BufReader::new(data);
            let real_sum = adler32(&mut buf).unwrap();
            assert_eq!(checksum, real_sum);
        }

        test(&[], 1);
        test(&[0; 1024], 0x04000001);
        test(&[0; 1024 * 1024], 0x00f00001);
        test(&[0xA5; 1024 * 1024], 0xd5009ab1);
    }
}
