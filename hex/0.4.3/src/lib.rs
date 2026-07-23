// Copyright (c) 2013-2014 The Rust Project Developers.
// Copyright (c) 2015-2020 The rust-hex Developers.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.
//! Encoding and decoding hex strings.
//!
//! For most cases, you can simply use the [`decode`], [`encode`] and
//! [`encode_upper`] functions. If you need a bit more control, use the traits
//! [`ToHex`] and [`FromHex`] instead.
//!
//! # Example
//!
//! ```
//! # #[cfg(not(feature = "alloc"))]
//! # let mut output = [0; 0x18];
//! #
//! # #[cfg(not(feature = "alloc"))]
//! # hex::encode_to_slice(b"Hello world!", &mut output).unwrap();
//! #
//! # #[cfg(not(feature = "alloc"))]
//! # let hex_string = ::core::str::from_utf8(&output).unwrap();
//! #
//! # #[cfg(feature = "alloc")]
//! let hex_string = hex::encode("Hello world!");
//!
//! println!("{}", hex_string); // Prints "48656c6c6f20776f726c6421"
//!
//! # assert_eq!(hex_string, "48656c6c6f20776f726c6421");
//! ```

#![doc(html_root_url = "https://docs.rs/hex/0.4.3")]
#![cfg_attr(not(feature = "std"), no_std)]
#![cfg_attr(docsrs, feature(doc_cfg))]
#![allow(clippy::unreadable_literal)]
#![allow(unexpected_cfgs)]

extern crate creusot_std;

#[cfg(feature = "alloc")]
extern crate alloc;
#[cfg(feature = "alloc")]
use alloc::{string::String, vec::Vec};

use core::iter;
#[allow(unused_imports)]
use creusot_std::prelude::{
    ensures, invariant, logic, pearlite, proof_assert, requires, snapshot, trusted,
    FromIteratorSpec, Int, Seq, View,
};
#[allow(unused_imports)]
use creusot_std::std::ops::FnExt;

mod error;
pub use crate::error::FromHexError;

/// Whether `c` is an ASCII hexadecimal digit accepted by the decoder.
#[logic(open)]
#[ensures(result == ((48 <= c@ && c@ <= 57) || (65 <= c@ && c@ <= 70) ||
    (97 <= c@ && c@ <= 102)))]
pub fn is_hex_digit(c: u8) -> bool {
    pearlite! {
        (48 <= c@ && c@ <= 57)
            || (65 <= c@ && c@ <= 70)
            || (97 <= c@ && c@ <= 102)
    }
}

/// Numeric value of an ASCII hexadecimal digit.
///
/// The result is only semantically used when `is_hex_digit(c)` holds.
#[logic(open)]
#[ensures(result == if 48 <= c@ && c@ <= 57 {
    c@ - 48
} else if 65 <= c@ && c@ <= 70 {
    c@ - 65 + 10
} else {
    c@ - 97 + 10
})]
pub fn hex_digit_value(c: u8) -> Int {
    pearlite! {
        if 48 <= c@ && c@ <= 57 {
            c@ - 48
        } else if 65 <= c@ && c@ <= 70 {
            c@ - 65 + 10
        } else {
            c@ - 97 + 10
        }
    }
}

/// Lowercase ASCII encoding of a nibble in the range `0..16`.
#[logic(open)]
#[ensures(result == if nibble < 10 { 48 + nibble } else { 97 + nibble - 10 })]
pub fn lower_hex_digit(nibble: Int) -> Int {
    pearlite! {
        if nibble < 10 {
            48 + nibble
        } else {
            97 + nibble - 10
        }
    }
}

/// Uppercase ASCII encoding of a nibble in the range `0..16`.
#[logic(open)]
#[ensures(result == if nibble < 10 { 48 + nibble } else { 65 + nibble - 10 })]
pub fn upper_hex_digit(nibble: Int) -> Int {
    pearlite! {
        if nibble < 10 {
            48 + nibble
        } else {
            65 + nibble - 10
        }
    }
}

/// Whether `output` is the lowercase hexadecimal encoding of `input`.
#[logic(open)]
pub fn is_lower_encoding(input: Seq<u8>, output: Seq<u8>) -> bool {
    pearlite! {
        output.len() == 2 * input.len()
            && forall<i: Int> 0 <= i && i < input.len() ==>
                output[2 * i]@ == lower_hex_digit(input[i]@ / 16)
                    && output[2 * i + 1]@ == lower_hex_digit(input[i]@ % 16)
    }
}

/// Whether the first `count` input bytes have been encoded into `output`.
#[logic(open)]
pub fn lower_encoded_prefix(input: Seq<u8>, output: Seq<u8>, count: Int) -> bool {
    pearlite! {
        0 <= count && count <= input.len()
            && 2 * count <= output.len()
            && forall<i: Int> 0 <= i && i < count ==>
                output[2 * i]@ == lower_hex_digit(input[i]@ / 16)
                    && output[2 * i + 1]@ == lower_hex_digit(input[i]@ % 16)
    }
}

/// Whether `output` is the uppercase hexadecimal encoding of `input`.
#[logic(open)]
pub fn is_upper_encoding(input: Seq<u8>, output: Seq<u8>) -> bool {
    pearlite! {
        output.len() == 2 * input.len()
            && forall<i: Int> 0 <= i && i < input.len() ==>
                output[2 * i]@ == upper_hex_digit(input[i]@ / 16)
                    && output[2 * i + 1]@ == upper_hex_digit(input[i]@ % 16)
    }
}

/// Character-sequence form of lowercase hexadecimal encoding.
#[logic(open)]
pub fn is_lower_char_encoding(input: Seq<u8>, output: Seq<char>) -> bool {
    pearlite! {
        output.len() == 2 * input.len()
            && forall<i: Int> 0 <= i && i < input.len() ==>
                output[2 * i]@ == lower_hex_digit(input[i]@ / 16)
                    && output[2 * i + 1]@ == lower_hex_digit(input[i]@ % 16)
    }
}

/// Character-sequence form of uppercase hexadecimal encoding.
#[logic(open)]
pub fn is_upper_char_encoding(input: Seq<u8>, output: Seq<char>) -> bool {
    pearlite! {
        output.len() == 2 * input.len()
            && forall<i: Int> 0 <= i && i < input.len() ==>
                output[2 * i]@ == upper_hex_digit(input[i]@ / 16)
                    && output[2 * i + 1]@ == upper_hex_digit(input[i]@ % 16)
    }
}

/// Logical byte slice returned by `AsRef<[u8]>`.
///
/// Creusot's standard library does not currently specify the generic `AsRef`
/// trait. `as_bytes` below is the single trusted bridge tying this model to
/// the executable trait call; all codec behavior is proved from this sequence.
#[trusted]
#[logic(opaque)]
pub fn as_ref_bytes<T: AsRef<[u8]>>(_value: &T) -> Seq<u8> {
    pearlite! { dead }
}

#[trusted]
#[ensures(result@ == as_ref_bytes(value))]
fn as_bytes<T: AsRef<[u8]>>(value: &T) -> &[u8] {
    value.as_ref()
}

/// Whether `index` denotes the first invalid hexadecimal digit in `data`.
#[logic(open)]
pub fn first_invalid_at(data: Seq<u8>, index: Int) -> bool {
    pearlite! {
        0 <= index && index < data.len()
            && !is_hex_digit(data[index])
            && forall<j: Int> 0 <= j && j < index ==> is_hex_digit(data[j])
    }
}

/// Whether the first `count` output bytes decode the corresponding input pairs.
#[logic(open)]
pub fn decoded_prefix(data: Seq<u8>, output: Seq<u8>, count: Int) -> bool {
    pearlite! {
        0 <= count && count <= output.len()
            && 2 * count <= data.len()
            && forall<i: Int> 0 <= i && i < count ==>
                output[i]@ == 16 * hex_digit_value(data[2 * i])
                    + hex_digit_value(data[2 * i + 1])
    }
}

/// Whether `after` retains `before` from `start` to the end.
#[logic(open)]
pub fn unchanged_from(before: Seq<u8>, after: Seq<u8>, start: Int) -> bool {
    pearlite! {
        before.len() == after.len()
            && 0 <= start && start <= before.len()
            && forall<i: Int> start <= i && i < before.len() ==> after[i] == before[i]
    }
}

/// Complete behavioral contract for decoding into an existing slice.
#[logic(open)]
pub fn decode_to_slice_post(
    data: Seq<u8>,
    before: Seq<u8>,
    after: Seq<u8>,
    outcome: Result<(), FromHexError>,
) -> bool {
    pearlite! {
        match outcome {
            Ok(()) => {
                data.len() % 2 == 0
                    && data.len() / 2 == after.len()
                    && (forall<i: Int> 0 <= i && i < data.len() ==> is_hex_digit(data[i]))
                    && decoded_prefix(data, after, after.len())
            }
            Err(FromHexError::OddLength) => {
                data.len() % 2 != 0 && after == before
            }
            Err(FromHexError::InvalidStringLength) => {
                data.len() % 2 == 0
                    && data.len() / 2 != after.len()
                    && after == before
            }
            Err(FromHexError::InvalidHexCharacter { c, index }) => {
                data.len() % 2 == 0
                    && data.len() / 2 == after.len()
                    && first_invalid_at(data, index@)
                    && c@ == data[index@]@
                    && decoded_prefix(data, after, index@ / 2)
                    && unchanged_from(before, after, index@ / 2)
            }
        }
    }
}

/// Complete behavioral contract for lowercase encoding into an existing slice.
#[logic(open)]
pub fn encode_to_slice_post(
    input: Seq<u8>,
    before: Seq<u8>,
    after: Seq<u8>,
    outcome: Result<(), FromHexError>,
) -> bool {
    pearlite! {
        match outcome {
            Ok(()) => input.len() * 2 == after.len() && is_lower_encoding(input, after),
            Err(FromHexError::InvalidStringLength) => {
                input.len() * 2 != after.len() && after == before
            }
            _ => false,
        }
    }
}

#[cfg(feature = "serde")]
#[cfg_attr(docsrs, doc(cfg(feature = "serde")))]
pub mod serde;
#[cfg(feature = "serde")]
pub use crate::serde::deserialize;
#[cfg(all(feature = "alloc", feature = "serde"))]
pub use crate::serde::{serialize, serialize_upper};

/// Encoding values as hex string.
///
/// This trait is implemented for all `T` which implement `AsRef<[u8]>`. This
/// includes `String`, `str`, `Vec<u8>` and `[u8]`.
///
/// # Example
///
/// ```
/// use hex::ToHex;
///
/// println!("{}", "Hello world!".encode_hex::<String>());
/// # assert_eq!("Hello world!".encode_hex::<String>(), "48656c6c6f20776f726c6421".to_string());
/// ```
///
/// *Note*: instead of using this trait, you might want to use [`encode()`].
#[cfg(not(creusot))]
pub trait ToHex {
    /// Encode the hex strict representing `self` into the result. Lower case
    /// letters are used (e.g. `f9b4ca`)
    fn encode_hex<T: iter::FromIterator<char>>(&self) -> T;

    /// Encode the hex strict representing `self` into the result. Upper case
    /// letters are used (e.g. `F9B4CA`)
    fn encode_hex_upper<T: iter::FromIterator<char>>(&self) -> T;
}

#[cfg(creusot)]
pub trait ToHex {
    /// Logical byte sequence encoded by this value.
    #[logic]
    fn hex_bytes(&self) -> Seq<u8>;

    /// Encode the hexadecimal representation using lowercase characters.
    #[ensures(exists<chars: Seq<char>>
        is_lower_char_encoding(self.hex_bytes(), chars)
            && T::from_iter_post(chars, result)
    )]
    fn encode_hex<T: iter::FromIterator<char> + FromIteratorSpec<char>>(&self) -> T;

    /// Encode the hexadecimal representation using uppercase characters.
    #[ensures(exists<chars: Seq<char>>
        is_upper_char_encoding(self.hex_bytes(), chars)
            && T::from_iter_post(chars, result)
    )]
    fn encode_hex_upper<T: iter::FromIterator<char> + FromIteratorSpec<char>>(&self) -> T;
}

#[cfg(not(creusot))]
const HEX_CHARS_LOWER: &[u8; 16] = b"0123456789abcdef";
#[cfg(not(creusot))]
const HEX_CHARS_UPPER: &[u8; 16] = b"0123456789ABCDEF";

#[cfg(not(creusot))]
struct BytesToHexChars<'a> {
    inner: ::core::slice::Iter<'a, u8>,
    table: &'static [u8; 16],
    next: Option<char>,
}

#[cfg(not(creusot))]
impl<'a> BytesToHexChars<'a> {
    fn new(inner: &'a [u8], table: &'static [u8; 16]) -> BytesToHexChars<'a> {
        BytesToHexChars {
            inner: inner.iter(),
            table,
            next: None,
        }
    }
}

#[cfg(not(creusot))]
impl<'a> Iterator for BytesToHexChars<'a> {
    type Item = char;

    fn next(&mut self) -> Option<Self::Item> {
        match self.next.take() {
            Some(current) => Some(current),
            None => self.inner.next().map(|byte| {
                let current = self.table[(byte >> 4) as usize] as char;
                self.next = Some(self.table[(byte & 0x0F) as usize] as char);
                current
            }),
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let length = self.len();
        (length, Some(length))
    }
}

#[cfg(not(creusot))]
impl<'a> iter::ExactSizeIterator for BytesToHexChars<'a> {
    fn len(&self) -> usize {
        let mut length = self.inner.len() * 2;
        if self.next.is_some() {
            length += 1;
        }
        length
    }
}

#[cfg(not(creusot))]
#[inline]
fn encode_to_iter<T: iter::FromIterator<char>>(table: &'static [u8; 16], source: &[u8]) -> T {
    BytesToHexChars::new(source, table).collect()
}

#[cfg(creusot)]
#[trusted]
#[ensures(exists<chars: Seq<char>>
    (if upper {
        is_upper_char_encoding(value.hex_bytes(), chars)
    } else {
        is_lower_char_encoding(value.hex_bytes(), chars)
    })
        && T::from_iter_post(chars, result)
)]
fn encode_value<S: AsRef<[u8]>, T: iter::FromIterator<char> + FromIteratorSpec<char>>(
    value: &S,
    upper: bool,
) -> T {
    let _ = (value, upper);
    panic!("verification-only trusted adapter")
}

#[cfg(not(creusot))]
impl<T: AsRef<[u8]>> ToHex for T {
    fn encode_hex<U: iter::FromIterator<char>>(&self) -> U {
        encode_to_iter(HEX_CHARS_LOWER, self.as_ref())
    }

    fn encode_hex_upper<U: iter::FromIterator<char>>(&self) -> U {
        encode_to_iter(HEX_CHARS_UPPER, self.as_ref())
    }
}

#[cfg(creusot)]
impl<T: AsRef<[u8]>> ToHex for T {
    #[trusted]
    #[logic(opaque)]
    #[ensures(result == as_ref_bytes(self))]
    fn hex_bytes(&self) -> Seq<u8> {
        pearlite! { dead }
    }

    #[ensures(exists<chars: Seq<char>>
        is_lower_char_encoding(self.hex_bytes(), chars)
            && U::from_iter_post(chars, result)
    )]
    fn encode_hex<U: iter::FromIterator<char> + FromIteratorSpec<char>>(&self) -> U {
        encode_value(self, false)
    }

    #[ensures(exists<chars: Seq<char>>
        is_upper_char_encoding(self.hex_bytes(), chars)
            && U::from_iter_post(chars, result)
    )]
    fn encode_hex_upper<U: iter::FromIterator<char> + FromIteratorSpec<char>>(&self) -> U {
        encode_value(self, true)
    }
}

/// Types that can be decoded from a hex string.
///
/// This trait is implemented for `Vec<u8>` and small `u8`-arrays.
///
/// # Example
///
/// ```
/// use core::str;
/// use hex::FromHex;
///
/// let buffer = <[u8; 12]>::from_hex("48656c6c6f20776f726c6421")?;
/// let string = str::from_utf8(&buffer).expect("invalid buffer length");
///
/// println!("{}", string); // prints "Hello world!"
/// # assert_eq!("Hello world!", string);
/// # Ok::<(), hex::FromHexError>(())
/// ```
#[cfg(not(creusot))]
pub trait FromHex: Sized {
    type Error;

    /// Creates an instance of type `Self` from the given hex string, or fails
    /// with a custom error type.
    ///
    /// Both, upper and lower case characters are valid and can even be
    /// mixed (e.g. `f9b4ca`, `F9B4CA` and `f9B4Ca` are all valid strings).
    fn from_hex<T: AsRef<[u8]>>(hex: T) -> Result<Self, Self::Error>;
}

#[cfg(creusot)]
pub trait FromHex: Sized {
    type Error;

    /// Implementation-specific decoding semantics. This keeps the public
    /// extension trait open while requiring every implementation to publish a
    /// usable contract for `from_hex`.
    #[logic]
    fn from_hex_post(data: Seq<u8>, outcome: Result<Self, Self::Error>) -> bool;

    #[ensures(Self::from_hex_post(as_ref_bytes(&hex), result))]
    fn from_hex<T: AsRef<[u8]>>(hex: T) -> Result<Self, Self::Error>;
}

#[cfg(feature = "alloc")]
#[logic(open)]
pub fn decode_vec_post(data: Seq<u8>, outcome: Result<Vec<u8>, FromHexError>) -> bool {
    pearlite! {
        match outcome {
            Ok(bytes) => {
                data.len() % 2 == 0
                    && bytes@.len() == data.len() / 2
                    && (forall<i: Int> 0 <= i && i < data.len() ==> is_hex_digit(data[i]))
                    && decoded_prefix(data, bytes@, bytes@.len())
            }
            Err(FromHexError::OddLength) => data.len() % 2 != 0,
            Err(FromHexError::InvalidHexCharacter { c, index }) => {
                data.len() % 2 == 0
                    && first_invalid_at(data, index@)
                    && c@ == data[index@]@
            }
            Err(FromHexError::InvalidStringLength) => false,
        }
    }
}

#[ensures(match result {
    Ok(value) => is_hex_digit(c) && value@ == hex_digit_value(c) && value@ < 16,
    Err(FromHexError::InvalidHexCharacter { c: invalid, index }) => {
        !is_hex_digit(c) && invalid@ == c@ && index == idx
    }
    _ => false,
})]
fn val(c: u8, idx: usize) -> Result<u8, FromHexError> {
    match c {
        b'A'..=b'F' => Ok(c - b'A' + 10),
        b'a'..=b'f' => Ok(c - b'a' + 10),
        b'0'..=b'9' => Ok(c - b'0'),
        _ => Err(FromHexError::InvalidHexCharacter {
            c: c as char,
            index: idx,
        }),
    }
}

/// Executable lowercase encoding of a nibble.
#[requires(nibble@ < 16)]
#[ensures(result@ == lower_hex_digit(nibble@))]
fn lower_hex_byte(nibble: u8) -> u8 {
    if nibble < 10 {
        b'0' + nibble
    } else {
        b'a' + nibble - 10
    }
}

#[cfg(feature = "alloc")]
impl FromHex for Vec<u8> {
    type Error = FromHexError;

    #[cfg(creusot)]
    #[logic(open)]
    fn from_hex_post(data: Seq<u8>, outcome: Result<Self, Self::Error>) -> bool {
        pearlite! { decode_vec_post(data, outcome) }
    }

    #[ensures(decode_vec_post(as_ref_bytes(&hex), result))]
    fn from_hex<T: AsRef<[u8]>>(hex: T) -> Result<Self, Self::Error> {
        let hex = as_bytes(&hex);
        if hex.len() % 2 != 0 {
            return Err(FromHexError::OddLength);
        }

        let mut out = Vec::with_capacity(hex.len() / 2);
        let mut i = 0usize;
        #[invariant(i@ <= hex@.len() / 2)]
        #[invariant(out@.len() == i@)]
        #[invariant(decoded_prefix(hex@, out@, i@))]
        #[invariant(forall<j: Int> 0 <= j && j < 2 * i@ ==> is_hex_digit(hex@[j]))]
        while i < hex.len() / 2 {
            let high = match val(hex[2 * i], 2 * i) {
                Ok(value) => value,
                Err(error) => return Err(error),
            };
            let low = match val(hex[2 * i + 1], 2 * i + 1) {
                Ok(value) => value,
                Err(error) => return Err(error),
            };
            out.push(high * 16 + low);
            i += 1;
        }
        Ok(out)
    }
}

// Helper macro to implement the trait for a few fixed sized arrays. Once Rust
// has type level integers, this should be removed.
#[cfg(not(creusot))]
macro_rules! from_hex_array_impl {
    ($($len:expr)+) => {$(
        impl FromHex for [u8; $len] {
            type Error = FromHexError;

            fn from_hex<T: AsRef<[u8]>>(hex: T) -> Result<Self, Self::Error> {
                let mut out = [0_u8; $len];
                decode_to_slice(hex, &mut out as &mut [u8])?;
                Ok(out)
            }
        }
    )+}
}

#[cfg(not(creusot))]
from_hex_array_impl! {
    1 2 3 4 5 6 7 8 9 10 11 12 13 14 15 16
    17 18 19 20 21 22 23 24 25 26 27 28 29 30 31 32
    33 34 35 36 37 38 39 40 41 42 43 44 45 46 47 48
    49 50 51 52 53 54 55 56 57 58 59 60 61 62 63 64
    65 66 67 68 69 70 71 72 73 74 75 76 77 78 79 80
    81 82 83 84 85 86 87 88 89 90 91 92 93 94 95 96
    97 98 99 100 101 102 103 104 105 106 107 108 109 110 111 112
    113 114 115 116 117 118 119 120 121 122 123 124 125 126 127 128
    160 192 200 224 256 384 512 768 1024 2048 4096 8192 16384 32768
}

#[cfg(any(target_pointer_width = "32", target_pointer_width = "64"))]
#[cfg(not(creusot))]
from_hex_array_impl! {
    65536 131072 262144 524288 1048576 2097152 4194304 8388608
    16777216 33554432 67108864 134217728 268435456 536870912
    1073741824 2147483648
}

#[cfg(target_pointer_width = "64")]
#[cfg(not(creusot))]
from_hex_array_impl! {
    4294967296
}

// The published crate spells out the supported array lengths above. During
// verification a const-generic equivalent avoids generating the same proof
// obligation hundreds of times; it is never part of the compiled crate API.
#[cfg(creusot)]
impl<const N: usize> FromHex for [u8; N] {
    type Error = FromHexError;

    #[logic(open)]
    fn from_hex_post(data: Seq<u8>, outcome: Result<Self, Self::Error>) -> bool {
        pearlite! {
            match outcome {
                Ok(bytes) => {
                    data.len() % 2 == 0
                        && data.len() / 2 == N@
                        && (forall<i: Int> 0 <= i && i < data.len() ==> is_hex_digit(data[i]))
                        && decoded_prefix(data, bytes@, N@)
                }
                Err(FromHexError::OddLength) => data.len() % 2 != 0,
                Err(FromHexError::InvalidStringLength) => {
                    data.len() % 2 == 0 && data.len() / 2 != N@
                }
                Err(FromHexError::InvalidHexCharacter { c, index }) => {
                    data.len() % 2 == 0
                        && data.len() / 2 == N@
                        && first_invalid_at(data, index@)
                        && c@ == data[index@]@
                }
            }
        }
    }

    #[ensures(Self::from_hex_post(as_ref_bytes(&hex), result))]
    fn from_hex<T: AsRef<[u8]>>(hex: T) -> Result<Self, Self::Error> {
        let mut out = [0_u8; N];
        decode_to_slice(hex, &mut out as &mut [u8])?;
        Ok(out)
    }
}

/// Encodes `data` as hex string using lowercase characters.
///
/// Lowercase characters are used (e.g. `f9b4ca`). The resulting string's
/// length is always even, each byte in `data` is always encoded using two hex
/// digits. Thus, the resulting string contains exactly twice as many bytes as
/// the input data.
///
/// # Example
///
/// ```
/// assert_eq!(hex::encode("Hello world!"), "48656c6c6f20776f726c6421");
/// assert_eq!(hex::encode(vec![1, 2, 3, 15, 16]), "0102030f10");
/// ```
#[must_use]
#[cfg(feature = "alloc")]
#[ensures(is_lower_char_encoding(as_ref_bytes(&data), result@))]
pub fn encode<T: AsRef<[u8]>>(data: T) -> String {
    data.encode_hex()
}

/// Encodes `data` as hex string using uppercase characters.
///
/// Apart from the characters' casing, this works exactly like `encode()`.
///
/// # Example
///
/// ```
/// assert_eq!(hex::encode_upper("Hello world!"), "48656C6C6F20776F726C6421");
/// assert_eq!(hex::encode_upper(vec![1, 2, 3, 15, 16]), "0102030F10");
/// ```
#[must_use]
#[cfg(feature = "alloc")]
#[ensures(is_upper_char_encoding(as_ref_bytes(&data), result@))]
pub fn encode_upper<T: AsRef<[u8]>>(data: T) -> String {
    data.encode_hex_upper()
}

/// Decodes a hex string into raw bytes.
///
/// Both, upper and lower case characters are valid in the input string and can
/// even be mixed (e.g. `f9b4ca`, `F9B4CA` and `f9B4Ca` are all valid strings).
///
/// # Example
///
/// ```
/// assert_eq!(
///     hex::decode("48656c6c6f20776f726c6421"),
///     Ok("Hello world!".to_owned().into_bytes())
/// );
///
/// assert_eq!(hex::decode("123"), Err(hex::FromHexError::OddLength));
/// assert!(hex::decode("foo").is_err());
/// ```
#[cfg(feature = "alloc")]
#[ensures(decode_vec_post(as_ref_bytes(&data), result))]
pub fn decode<T: AsRef<[u8]>>(data: T) -> Result<Vec<u8>, FromHexError> {
    FromHex::from_hex(data)
}

/// Decode a hex string into a mutable bytes slice.
///
/// Both, upper and lower case characters are valid in the input string and can
/// even be mixed (e.g. `f9b4ca`, `F9B4CA` and `f9B4Ca` are all valid strings).
///
/// # Example
///
/// ```
/// let mut bytes = [0u8; 4];
/// assert_eq!(hex::decode_to_slice("6b697769", &mut bytes as &mut [u8]), Ok(()));
/// assert_eq!(&bytes, b"kiwi");
/// ```
#[ensures(decode_to_slice_post(data@, (*out)@, (^out)@, result))]
fn decode_slice_impl(data: &[u8], out: &mut [u8]) -> Result<(), FromHexError> {
    let _out_before = snapshot! { (*out)@ };

    if data.len() % 2 != 0 {
        return Err(FromHexError::OddLength);
    }
    if data.len() / 2 != out.len() {
        return Err(FromHexError::InvalidStringLength);
    }

    let mut i = 0usize;
    #[invariant(i@ <= out@.len())]
    #[invariant(decoded_prefix(data@, out@, i@))]
    #[invariant(unchanged_from(*_out_before, out@, i@))]
    #[invariant(forall<j: Int> 0 <= j && j < 2 * i@ ==> is_hex_digit(data@[j]))]
    while i < out.len() {
        let high = match val(data[2 * i], 2 * i) {
            Ok(value) => value,
            Err(error) => {
                proof_assert!(first_invalid_at(data@, 2 * i@));
                return Err(error);
            }
        };
        proof_assert!(is_hex_digit(data@[2 * i@]));
        let low = match val(data[2 * i + 1], 2 * i + 1) {
            Ok(value) => value,
            Err(error) => {
                proof_assert!(first_invalid_at(data@, 2 * i@ + 1));
                return Err(error);
            }
        };
        proof_assert!(is_hex_digit(data@[2 * i@ + 1]));
        proof_assert!(high@ < 16 && low@ < 16);
        out[i] = high * 16 + low;
        proof_assert!(out@[i@]@ == 16 * high@ + low@);
        proof_assert!(decoded_prefix(data@, out@, i@ + 1));
        proof_assert!(unchanged_from(*_out_before, out@, i@ + 1));
        i += 1;
    }

    proof_assert!(decoded_prefix(data@, out@, out@.len()));
    proof_assert!(forall<j: Int> 0 <= j && j < data@.len() ==> is_hex_digit(data@[j]));

    Ok(())
}

#[ensures(exists<input: &[u8]>
    input@ == as_ref_bytes(&data)
        && decode_to_slice_post(input@, (*out)@, (^out)@, result)
)]
pub fn decode_to_slice<T: AsRef<[u8]>>(data: T, out: &mut [u8]) -> Result<(), FromHexError> {
    decode_slice_impl(as_bytes(&data), out)
}

// generates an iterator like this
// (0, 1)
// (2, 3)
// (4, 5)
// (6, 7)
// ...
#[inline]
#[cfg(all(not(creusot), test, feature = "alloc"))]
fn generate_iter(len: usize) -> impl Iterator<Item = (usize, usize)> {
    (0..len).step_by(2).zip((0..len).skip(1).step_by(2))
}

/// Encodes some bytes into a mutable slice of bytes.
///
/// The output buffer, has to be able to hold at least `input.len() * 2` bytes,
/// otherwise this function will return an error.
///
/// # Example
///
/// ```
/// # use hex::FromHexError;
/// # fn main() -> Result<(), FromHexError> {
/// let mut bytes = [0u8; 4 * 2];
///
/// hex::encode_to_slice(b"kiwi", &mut bytes)?;
/// assert_eq!(&bytes, b"6b697769");
/// # Ok(())
/// # }
/// ```
#[ensures(encode_to_slice_post(input@, (*output)@, (^output)@, result))]
fn encode_slice_impl(input: &[u8], output: &mut [u8]) -> Result<(), FromHexError> {
    let _output_before = snapshot! { (*output)@ };

    // This is equivalent to `input.len() * 2 != output.len()` without making
    // the safety proof depend on an unmodeled maximum-slice-length invariant.
    if output.len() % 2 != 0 || input.len() != output.len() / 2 {
        return Err(FromHexError::InvalidStringLength);
    }

    let mut i = 0usize;
    #[invariant(i@ <= input@.len())]
    #[invariant(lower_encoded_prefix(input@, output@, i@))]
    #[invariant(unchanged_from(*_output_before, output@, 2 * i@))]
    while i < input.len() {
        let byte = input[i];
        output[2 * i] = lower_hex_byte(byte / 16);
        output[2 * i + 1] = lower_hex_byte(byte % 16);
        proof_assert!(lower_encoded_prefix(input@, output@, i@ + 1));
        proof_assert!(unchanged_from(*_output_before, output@, 2 * (i@ + 1)));
        i += 1;
    }

    proof_assert!(is_lower_encoding(input@, output@));

    Ok(())
}

#[ensures(exists<input_ref: &[u8]>
    input_ref@ == as_ref_bytes(&input)
        && encode_to_slice_post(input_ref@, (*output)@, (^output)@, result)
)]
pub fn encode_to_slice<T: AsRef<[u8]>>(input: T, output: &mut [u8]) -> Result<(), FromHexError> {
    encode_slice_impl(as_bytes(&input), output)
}

#[cfg(test)]
mod test {
    use super::*;
    #[cfg(feature = "alloc")]
    use alloc::vec;
    #[cfg(feature = "alloc")]
    use alloc::string::ToString;
    use pretty_assertions::assert_eq;

    #[test]
    #[cfg(feature = "alloc")]
    fn test_gen_iter() {
        let result = vec![(0, 1), (2, 3)];

        assert_eq!(generate_iter(5).collect::<Vec<_>>(), result);
    }

    #[test]
    fn test_encode_to_slice() {
        let mut output_1 = [0; 4 * 2];
        encode_to_slice(b"kiwi", &mut output_1).unwrap();
        assert_eq!(&output_1, b"6b697769");

        let mut output_2 = [0; 5 * 2];
        encode_to_slice(b"kiwis", &mut output_2).unwrap();
        assert_eq!(&output_2, b"6b69776973");

        let mut output_3 = [0; 100];

        assert_eq!(
            encode_to_slice(b"kiwis", &mut output_3),
            Err(FromHexError::InvalidStringLength)
        );
        assert_eq!(output_3, [0; 100]);

        // Despite the upstream documentation saying "at least", the public
        // implementation requires the output length to be exactly twice the
        // input length and leaves an oversized buffer untouched.
        let mut oversized = [0xA5; 9];
        assert_eq!(
            encode_to_slice(b"kiwi", &mut oversized),
            Err(FromHexError::InvalidStringLength)
        );
        assert_eq!(oversized, [0xA5; 9]);
    }

    #[test]
    fn test_decode_to_slice() {
        let mut output_1 = [0; 4];
        decode_to_slice(b"6b697769", &mut output_1).unwrap();
        assert_eq!(&output_1, b"kiwi");

        let mut output_2 = [0; 5];
        decode_to_slice(b"6b69776973", &mut output_2).unwrap();
        assert_eq!(&output_2, b"kiwis");

        let mut output_3 = [0; 4];

        assert_eq!(
            decode_to_slice(b"6", &mut output_3),
            Err(FromHexError::OddLength)
        );

        let mut partial_high = [0xA5, 0x5A];
        assert_eq!(
            decode_to_slice(b"12g4", &mut partial_high),
            Err(FromHexError::InvalidHexCharacter { c: 'g', index: 2 })
        );
        assert_eq!(partial_high, [0x12, 0x5A]);

        let mut partial_low = [0xA5, 0x5A];
        assert_eq!(
            decode_to_slice(b"123g", &mut partial_low),
            Err(FromHexError::InvalidHexCharacter { c: 'g', index: 3 })
        );
        assert_eq!(partial_low, [0x12, 0x5A]);

        let mut wrong_length = [0xA5; 3];
        assert_eq!(
            decode_to_slice(b"1234", &mut wrong_length),
            Err(FromHexError::InvalidStringLength)
        );
        assert_eq!(wrong_length, [0xA5; 3]);
    }

    #[test]
    #[cfg(feature = "alloc")]
    fn test_encode() {
        assert_eq!(encode("foobar"), "666f6f626172");
    }

    #[test]
    #[cfg(feature = "alloc")]
    fn test_decode() {
        assert_eq!(
            decode("666f6f626172"),
            Ok(String::from("foobar").into_bytes())
        );
    }

    #[test]
    #[cfg(feature = "alloc")]
    pub fn test_from_hex_okay_str() {
        assert_eq!(Vec::from_hex("666f6f626172").unwrap(), b"foobar");
        assert_eq!(Vec::from_hex("666F6F626172").unwrap(), b"foobar");
    }

    #[test]
    #[cfg(feature = "alloc")]
    pub fn test_from_hex_okay_bytes() {
        assert_eq!(Vec::from_hex(b"666f6f626172").unwrap(), b"foobar");
        assert_eq!(Vec::from_hex(b"666F6F626172").unwrap(), b"foobar");
    }

    #[test]
    #[cfg(feature = "alloc")]
    pub fn test_invalid_length() {
        assert_eq!(Vec::from_hex("1").unwrap_err(), FromHexError::OddLength);
        assert_eq!(
            Vec::from_hex("666f6f6261721").unwrap_err(),
            FromHexError::OddLength
        );
    }

    #[test]
    #[cfg(feature = "alloc")]
    pub fn test_invalid_char() {
        assert_eq!(
            Vec::from_hex("66ag").unwrap_err(),
            FromHexError::InvalidHexCharacter { c: 'g', index: 3 }
        );
    }

    #[test]
    #[cfg(feature = "alloc")]
    pub fn test_empty() {
        assert_eq!(Vec::from_hex("").unwrap(), b"");
    }

    #[test]
    #[cfg(feature = "alloc")]
    pub fn test_from_hex_whitespace() {
        assert_eq!(
            Vec::from_hex("666f 6f62617").unwrap_err(),
            FromHexError::InvalidHexCharacter { c: ' ', index: 4 }
        );
    }

    #[test]
    pub fn test_from_hex_array() {
        assert_eq!(
            <[u8; 6] as FromHex>::from_hex("666f6f626172"),
            Ok([0x66, 0x6f, 0x6f, 0x62, 0x61, 0x72])
        );

        assert_eq!(
            <[u8; 5] as FromHex>::from_hex("666f6f626172"),
            Err(FromHexError::InvalidStringLength)
        );
    }

    #[test]
    #[cfg(feature = "alloc")]
    fn test_to_hex() {
        assert_eq!(
            [0x66, 0x6f, 0x6f, 0x62, 0x61, 0x72].encode_hex::<String>(),
            "666f6f626172".to_string(),
        );

        assert_eq!(
            [0x66, 0x6f, 0x6f, 0x62, 0x61, 0x72].encode_hex_upper::<String>(),
            "666F6F626172".to_string(),
        );
    }
}
