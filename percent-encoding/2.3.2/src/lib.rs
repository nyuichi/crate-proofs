// Copyright 2013-2016 The rust-url developers.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! URLs use special characters to indicate the parts of the request.
//! For example, a `?` question mark marks the end of a path and the start of a query string.
//! In order for that character to exist inside a path, it needs to be encoded differently.
//!
//! Percent encoding replaces reserved characters with the `%` escape character
//! followed by a byte value as two hexadecimal digits.
//! For example, an ASCII space is replaced with `%20`.
//!
//! When encoding, the set of characters that can (and should, for readability) be left alone
//! depends on the context.
//! The `?` question mark mentioned above is not a separator when used literally
//! inside of a query string, and therefore does not need to be encoded.
//! The [`AsciiSet`] parameter of [`percent_encode`] and [`utf8_percent_encode`]
//! lets callers configure this.
//!
//! This crate deliberately does not provide many different sets.
//! Users should consider in what context the encoded string will be used,
//! read relevant specifications, and define their own set.
//! This is done by using the `add` method of an existing set.
//!
//! # Examples
//!
//! ```
//! use percent_encoding::{utf8_percent_encode, AsciiSet, CONTROLS};
//!
//! /// https://url.spec.whatwg.org/#fragment-percent-encode-set
//! const FRAGMENT: &AsciiSet = &CONTROLS.add(b' ').add(b'"').add(b'<').add(b'>').add(b'`');
//!
//! assert_eq!(utf8_percent_encode("foo <bar>", FRAGMENT).to_string(), "foo%20%3Cbar%3E");
//! ```
#![no_std]

extern crate creusot_std;

#[cfg(creusot)]
use creusot_std::prelude::resolve;
#[allow(unused_imports)]
use creusot_std::prelude::{
    ensures, invariant, logic, pearlite, proof_assert, requires, snapshot, variant, DeepModel, Int,
    Invariant, IteratorSpec, Seq, View,
};

// For forwards compatibility
#[cfg(feature = "std")]
extern crate std as _;

#[cfg(feature = "alloc")]
extern crate alloc;

#[cfg(feature = "alloc")]
use alloc::{
    borrow::{Cow, ToOwned},
    string::String,
    vec::Vec,
};
#[allow(unused_imports)]
use core::{fmt, slice, str};
#[cfg(all(feature = "alloc", creusot))]
use creusot_std::std::cow::{CowBytesExt, CowStrExt};
#[cfg(all(feature = "alloc", creusot))]
use creusot_std::std::string::{utf8_error_matches, utf8_lossy_model, valid_utf8, Utf8ErrorExt};

pub use self::ascii_set::{AsciiSet, CONTROLS, NON_ALPHANUMERIC};

mod ascii_set;

/// Membership in the logical four-chunk representation of an [`AsciiSet`].
#[logic(open)]
#[requires(mask.len() == 4)]
#[requires(byte@ < 0x80)]
pub fn ascii_set_contains(mask: Seq<u32>, byte: u8) -> bool {
    pearlite! {
        let chunk = if byte < 32u8 {
            mask[0]
        } else if byte < 64u8 {
            mask[1]
        } else if byte < 96u8 {
            mask[2]
        } else {
            mask[3]
        };
        let bit = if byte < 32u8 {
            byte
        } else if byte < 64u8 {
            byte - 32u8
        } else if byte < 96u8 {
            byte - 64u8
        } else {
            byte - 96u8
        };
        (chunk & (1u32 << bit)) != 0u32
    }
}

/// Exact representation update for inserting one ASCII byte.
#[logic(open)]
#[requires(mask.len() == 4)]
#[requires(byte@ < 0x80)]
pub fn ascii_set_add_model(mask: Seq<u32>, byte: u8) -> Seq<u32> {
    if byte < 32u8 {
        pearlite! { mask.set(0, mask[0] | (1u32 << byte)) }
    } else if byte < 64u8 {
        pearlite! { mask.set(1, mask[1] | (1u32 << (byte - 32u8))) }
    } else if byte < 96u8 {
        pearlite! { mask.set(2, mask[2] | (1u32 << (byte - 64u8))) }
    } else {
        pearlite! { mask.set(3, mask[3] | (1u32 << (byte - 96u8))) }
    }
}

/// Exact representation update for removing one ASCII byte.
#[logic(open)]
#[requires(mask.len() == 4)]
#[requires(byte@ < 0x80)]
pub fn ascii_set_remove_model(mask: Seq<u32>, byte: u8) -> Seq<u32> {
    if byte < 32u8 {
        pearlite! { mask.set(0, mask[0] & !(1u32 << byte)) }
    } else if byte < 64u8 {
        pearlite! { mask.set(1, mask[1] & !(1u32 << (byte - 32u8))) }
    } else if byte < 96u8 {
        pearlite! { mask.set(2, mask[2] & !(1u32 << (byte - 64u8))) }
    } else {
        pearlite! { mask.set(3, mask[3] & !(1u32 << (byte - 96u8))) }
    }
}

/// Exact four-chunk union representation.
#[logic(open)]
#[requires(left.len() == 4 && right.len() == 4)]
pub fn ascii_set_union_model(left: Seq<u32>, right: Seq<u32>) -> Seq<u32> {
    pearlite! {
        Seq::singleton(left[0] | right[0])
            .push_back(left[1] | right[1])
            .push_back(left[2] | right[2])
            .push_back(left[3] | right[3])
    }
}

/// Exact four-chunk complement representation.
#[logic(open)]
#[requires(mask.len() == 4)]
pub fn ascii_set_complement_model(mask: Seq<u32>) -> Seq<u32> {
    pearlite! {
        Seq::singleton(!mask[0])
            .push_back(!mask[1])
            .push_back(!mask[2])
            .push_back(!mask[3])
    }
}

/// Numeric value of an uppercase hexadecimal digit used by percent encoding.
#[logic(open)]
#[requires(0 <= nibble && nibble < 16)]
#[ensures(result == if nibble < 10 { nibble + 48 } else { nibble + 55 })]
pub fn upper_hex_digit(nibble: Int) -> Int {
    if nibble < 10 {
        nibble + 48
    } else {
        nibble + 55
    }
}

/// Exact three-byte `%HH` representation of one byte.
#[logic(open)]
pub fn encoded_byte_model(byte: u8) -> Seq<Int> {
    pearlite! {
        Seq::singleton(37)
            .push_back(upper_hex_digit(byte@ / 16))
            .push_back(upper_hex_digit(byte@ % 16))
    }
}

/// Lift a concrete byte sequence into mathematical integers.
#[logic(open)]
#[variant(bytes.len())]
#[ensures(bytes.len() == 0 ==> result == Seq::empty())]
#[ensures(bytes.len() == 1 ==> result == Seq::singleton(bytes[0]@))]
pub fn byte_values(bytes: Seq<u8>) -> Seq<Int> {
    if bytes.len() == 0 {
        Seq::empty()
    } else {
        pearlite! {
            Seq::singleton(bytes[0]@).concat(byte_values(bytes.tail()))
        }
    }
}

#[logic]
#[variant(left.len())]
#[ensures(byte_values(left.concat(right)) == byte_values(left).concat(byte_values(right)))]
pub fn byte_values_concat(left: Seq<u8>, right: Seq<u8>) {
    if left.len() > 0 {
        byte_values_concat(left.tail(), right)
    }
}

/// Concatenated byte values of chunks already yielded by an encoder.
#[logic(open)]
#[variant(chunks.len())]
#[ensures(chunks.len() == 0 ==> result == Seq::empty())]
#[ensures(chunks.len() == 1 ==>
    result == byte_values((*chunks[0])@.to_bytes()))]
pub fn encoded_chunk_values<'a>(chunks: Seq<&'a str>) -> Seq<Int> {
    if chunks.len() == 0 {
        Seq::empty()
    } else {
        pearlite! {
            byte_values((*chunks[0])@.to_bytes())
                .concat(encoded_chunk_values(chunks.tail()))
        }
    }
}

#[logic]
#[variant(left.len())]
#[ensures(encoded_chunk_values(left.concat(right)) ==
    encoded_chunk_values(left).concat(encoded_chunk_values(right)))]
pub fn encoded_chunk_values_concat<'a>(left: Seq<&'a str>, right: Seq<&'a str>) {
    if left.len() > 0 {
        encoded_chunk_values_concat(left.tail(), right)
    }
}

/// Compose two already-flattened encoder prefixes without exposing iterator
/// protocol details to the sequence-algebra proof.
#[logic]
#[requires(a == encoded_chunk_values(ab).concat(b))]
#[requires(b == encoded_chunk_values(bc).concat(c))]
#[ensures(a == encoded_chunk_values(ab.concat(bc)).concat(c))]
pub fn encoded_chunk_values_trans<'a>(
    a: Seq<Int>,
    ab: Seq<&'a str>,
    b: Seq<Int>,
    bc: Seq<&'a str>,
    c: Seq<Int>,
) {
    encoded_chunk_values_concat(ab, bc);
}

/// Concatenating an unescaped prefix contributes that prefix unchanged to the
/// complete percent-encoding model.
#[logic]
#[requires(mask.len() == 4)]
#[requires(forall<i> 0 <= i && i < prefix.len() ==>
    !should_encode_model(mask, prefix[i]))]
#[variant(prefix.len())]
#[ensures(percent_encode_model(prefix.concat(suffix), mask)
    == byte_values(prefix).concat(percent_encode_model(suffix, mask)))]
pub fn percent_encode_unencoded_concat(prefix: Seq<u8>, suffix: Seq<u8>, mask: Seq<u32>) {
    if prefix.len() > 0 {
        proof_assert!(!should_encode_model(mask, prefix[0]));
        proof_assert!(forall<i> 0 <= i && i < prefix.tail().len() ==>
            prefix.tail()[i] == prefix[i + 1]);
        proof_assert!(forall<i> 0 <= i && i < prefix.tail().len() ==>
            !should_encode_model(mask, prefix.tail()[i]));
        percent_encode_unencoded_concat(prefix.tail(), suffix, mask);
        proof_assert!(prefix.concat(suffix).tail().len() == prefix.tail().concat(suffix).len());
        proof_assert!(forall<i> 0 <= i && i < prefix.concat(suffix).tail().len() ==>
            prefix.concat(suffix).tail()[i] == prefix.tail().concat(suffix)[i]);
        proof_assert!(prefix.concat(suffix).tail() == prefix.tail().concat(suffix));
        proof_assert!(percent_encode_model(prefix.concat(suffix), mask)
        == Seq::singleton(prefix[0]@).concat(
            percent_encode_model(prefix.tail().concat(suffix), mask),
        ));
        proof_assert!(byte_values(prefix)
            == Seq::singleton(prefix[0]@).concat(byte_values(prefix.tail())));
        proof_assert!(forall<x: Seq<Int>, y: Seq<Int>, z: Seq<Int>>
            x.concat(y.concat(z)) == x.concat(y).concat(z));
        proof_assert!(Seq::singleton(prefix[0]@).concat(
            byte_values(prefix.tail()).concat(percent_encode_model(suffix, mask)),
        ) == Seq::singleton(prefix[0]@).concat(byte_values(prefix.tail()))
            .concat(percent_encode_model(suffix, mask)));
        proof_assert!(percent_encode_model(prefix.concat(suffix), mask)
        == Seq::singleton(prefix[0]@).concat(
            byte_values(prefix.tail()).concat(percent_encode_model(suffix, mask)),
        ));
        proof_assert!(byte_values(prefix).concat(percent_encode_model(suffix, mask))
        == Seq::singleton(prefix[0]@).concat(
            byte_values(prefix.tail()).concat(percent_encode_model(suffix, mask)),
        ));
    } else {
        proof_assert!(prefix == Seq::empty());
        proof_assert!(prefix.concat(suffix) == suffix);
        proof_assert!(byte_values(prefix) == Seq::empty());
        proof_assert!(
            Seq::<Int>::empty().concat(percent_encode_model(suffix, mask))
                == percent_encode_model(suffix, mask)
        );
        proof_assert!(
            percent_encode_model(prefix.concat(suffix), mask)
                == byte_values(prefix).concat(percent_encode_model(suffix, mask))
        );
    }
}

#[logic(open)]
#[requires(mask.len() == 4)]
pub fn should_encode_model(mask: Seq<u32>, byte: u8) -> bool {
    pearlite! { byte@ >= 0x80 || ascii_set_contains(mask, byte) }
}

/// Complete percent encoding of an input byte sequence.
#[logic(open)]
#[requires(mask.len() == 4)]
#[variant(bytes.len())]
#[ensures(forall<i> 0 <= i && i < result.len() ==> 0 <= result[i] && result[i] < 0x80)]
pub fn percent_encode_model(bytes: Seq<u8>, mask: Seq<u32>) -> Seq<Int> {
    if bytes.len() == 0 {
        Seq::empty()
    } else if should_encode_model(mask, bytes[0]) {
        pearlite! {
            encoded_byte_model(bytes[0]).concat(percent_encode_model(bytes.tail(), mask))
        }
    } else {
        pearlite! {
            Seq::singleton(bytes[0]@).concat(percent_encode_model(bytes.tail(), mask))
        }
    }
}

#[logic]
#[requires(mask.len() == 4)]
#[variant(bytes.len())]
pub fn no_percent_encoding(bytes: Seq<u8>, mask: Seq<u32>) -> bool {
    if bytes.len() == 0 {
        true
    } else {
        pearlite! {
            !should_encode_model(mask, bytes[0])
                && no_percent_encoding(bytes.tail(), mask)
        }
    }
}

#[logic(open)]
#[requires(mask.len() == 4)]
pub fn percent_encode_borrows(bytes: Seq<u8>, mask: Seq<u32>) -> bool {
    pearlite! {
        bytes.len() == 0
            || no_percent_encoding(bytes, mask)
            || (bytes.len() == 1 && should_encode_model(mask, bytes[0]))
    }
}

#[logic(open)]
pub fn is_hex_digit_model(byte: u8) -> bool {
    pearlite! {
        (48 <= byte@ && byte@ <= 57)
            || (65 <= byte@ && byte@ <= 70)
            || (97 <= byte@ && byte@ <= 102)
    }
}

#[logic(open)]
#[requires(is_hex_digit_model(byte))]
pub fn hex_value_model(byte: u8) -> Int {
    pearlite! {
        if 48 <= byte@ && byte@ <= 57 {
            byte@ - 48
        } else if 65 <= byte@ && byte@ <= 70 {
            byte@ - 55
        } else {
            byte@ - 87
        }
    }
}

/// WHATWG percent decoding: only `%` followed by two hexadecimal digits is consumed.
#[logic]
#[variant(bytes.len())]
#[ensures(forall<i> 0 <= i && i < result.len() ==> 0 <= result[i] && result[i] <= 0xff)]
pub fn percent_decode_model(bytes: Seq<u8>) -> Seq<Int> {
    if bytes.len() >= 3
        && bytes[0] == 37u8
        && is_hex_digit_model(bytes[1])
        && is_hex_digit_model(bytes[2])
    {
        pearlite! {
            Seq::singleton(hex_value_model(bytes[1]) * 16 + hex_value_model(bytes[2]))
                .concat(percent_decode_model(bytes.subsequence(3, bytes.len())))
        }
    } else if bytes.len() > 0 {
        pearlite! {
            Seq::singleton(bytes[0]@).concat(percent_decode_model(bytes.tail()))
        }
    } else {
        Seq::empty()
    }
}

#[logic]
#[variant(bytes.len())]
pub fn has_percent_escape(bytes: Seq<u8>) -> bool {
    if bytes.len() >= 3
        && bytes[0] == 37u8
        && is_hex_digit_model(bytes[1])
        && is_hex_digit_model(bytes[2])
    {
        true
    } else if bytes.len() > 0 {
        pearlite! { has_percent_escape(bytes.tail()) }
    } else {
        false
    }
}

/// If no valid `%HH` escape occurs, decoding preserves every input byte.
#[logic]
#[variant(bytes.len())]
#[requires(!has_percent_escape(bytes))]
#[ensures(percent_decode_model(bytes) == byte_values(bytes))]
pub fn percent_decode_identity(bytes: Seq<u8>) {
    if bytes.len() > 0 {
        proof_assert!(
            !(bytes.len() >= 3
                && bytes[0] == 37u8
                && is_hex_digit_model(bytes[1])
                && is_hex_digit_model(bytes[2]))
        );
        proof_assert!(!has_percent_escape(bytes.tail()));
        percent_decode_identity(bytes.tail());
    }
}

/// Return the percent-encoding of the given byte.
///
/// This is unconditional, unlike `percent_encode()` which has an `AsciiSet` parameter.
///
/// # Examples
///
/// ```
/// use percent_encoding::percent_encode_byte;
///
/// assert_eq!("foo bar".bytes().map(percent_encode_byte).collect::<String>(),
///            "%66%6F%6F%20%62%61%72");
/// ```
#[inline]
#[cfg(not(creusot))]
pub fn percent_encode_byte(byte: u8) -> &'static str {
    static ENC_TABLE: &[u8; 768] = b"\
      %00%01%02%03%04%05%06%07%08%09%0A%0B%0C%0D%0E%0F\
      %10%11%12%13%14%15%16%17%18%19%1A%1B%1C%1D%1E%1F\
      %20%21%22%23%24%25%26%27%28%29%2A%2B%2C%2D%2E%2F\
      %30%31%32%33%34%35%36%37%38%39%3A%3B%3C%3D%3E%3F\
      %40%41%42%43%44%45%46%47%48%49%4A%4B%4C%4D%4E%4F\
      %50%51%52%53%54%55%56%57%58%59%5A%5B%5C%5D%5E%5F\
      %60%61%62%63%64%65%66%67%68%69%6A%6B%6C%6D%6E%6F\
      %70%71%72%73%74%75%76%77%78%79%7A%7B%7C%7D%7E%7F\
      %80%81%82%83%84%85%86%87%88%89%8A%8B%8C%8D%8E%8F\
      %90%91%92%93%94%95%96%97%98%99%9A%9B%9C%9D%9E%9F\
      %A0%A1%A2%A3%A4%A5%A6%A7%A8%A9%AA%AB%AC%AD%AE%AF\
      %B0%B1%B2%B3%B4%B5%B6%B7%B8%B9%BA%BB%BC%BD%BE%BF\
      %C0%C1%C2%C3%C4%C5%C6%C7%C8%C9%CA%CB%CC%CD%CE%CF\
      %D0%D1%D2%D3%D4%D5%D6%D7%D8%D9%DA%DB%DC%DD%DE%DF\
      %E0%E1%E2%E3%E4%E5%E6%E7%E8%E9%EA%EB%EC%ED%EE%EF\
      %F0%F1%F2%F3%F4%F5%F6%F7%F8%F9%FA%FB%FC%FD%FE%FF\
      ";

    let index = usize::from(byte) * 3;
    // SAFETY: ENC_TABLE is ascii-only, so any subset of it should be
    // ascii-only too, which is valid utf8.
    unsafe { str::from_utf8_unchecked(&ENC_TABLE[index..index + 3]) }
}

#[cfg(creusot)]
#[inline]
#[creusot_std::prelude::trusted]
#[ensures(byte_values(result@.to_bytes()) == encoded_byte_model(byte))]
pub fn percent_encode_byte(byte: u8) -> &'static str {
    match byte {
        0 => "%00",
        1 => "%01",
        2 => "%02",
        3 => "%03",
        4 => "%04",
        5 => "%05",
        6 => "%06",
        7 => "%07",
        8 => "%08",
        9 => "%09",
        10 => "%0A",
        11 => "%0B",
        12 => "%0C",
        13 => "%0D",
        14 => "%0E",
        15 => "%0F",
        16 => "%10",
        17 => "%11",
        18 => "%12",
        19 => "%13",
        20 => "%14",
        21 => "%15",
        22 => "%16",
        23 => "%17",
        24 => "%18",
        25 => "%19",
        26 => "%1A",
        27 => "%1B",
        28 => "%1C",
        29 => "%1D",
        30 => "%1E",
        31 => "%1F",
        32 => "%20",
        33 => "%21",
        34 => "%22",
        35 => "%23",
        36 => "%24",
        37 => "%25",
        38 => "%26",
        39 => "%27",
        40 => "%28",
        41 => "%29",
        42 => "%2A",
        43 => "%2B",
        44 => "%2C",
        45 => "%2D",
        46 => "%2E",
        47 => "%2F",
        48 => "%30",
        49 => "%31",
        50 => "%32",
        51 => "%33",
        52 => "%34",
        53 => "%35",
        54 => "%36",
        55 => "%37",
        56 => "%38",
        57 => "%39",
        58 => "%3A",
        59 => "%3B",
        60 => "%3C",
        61 => "%3D",
        62 => "%3E",
        63 => "%3F",
        64 => "%40",
        65 => "%41",
        66 => "%42",
        67 => "%43",
        68 => "%44",
        69 => "%45",
        70 => "%46",
        71 => "%47",
        72 => "%48",
        73 => "%49",
        74 => "%4A",
        75 => "%4B",
        76 => "%4C",
        77 => "%4D",
        78 => "%4E",
        79 => "%4F",
        80 => "%50",
        81 => "%51",
        82 => "%52",
        83 => "%53",
        84 => "%54",
        85 => "%55",
        86 => "%56",
        87 => "%57",
        88 => "%58",
        89 => "%59",
        90 => "%5A",
        91 => "%5B",
        92 => "%5C",
        93 => "%5D",
        94 => "%5E",
        95 => "%5F",
        96 => "%60",
        97 => "%61",
        98 => "%62",
        99 => "%63",
        100 => "%64",
        101 => "%65",
        102 => "%66",
        103 => "%67",
        104 => "%68",
        105 => "%69",
        106 => "%6A",
        107 => "%6B",
        108 => "%6C",
        109 => "%6D",
        110 => "%6E",
        111 => "%6F",
        112 => "%70",
        113 => "%71",
        114 => "%72",
        115 => "%73",
        116 => "%74",
        117 => "%75",
        118 => "%76",
        119 => "%77",
        120 => "%78",
        121 => "%79",
        122 => "%7A",
        123 => "%7B",
        124 => "%7C",
        125 => "%7D",
        126 => "%7E",
        127 => "%7F",
        128 => "%80",
        129 => "%81",
        130 => "%82",
        131 => "%83",
        132 => "%84",
        133 => "%85",
        134 => "%86",
        135 => "%87",
        136 => "%88",
        137 => "%89",
        138 => "%8A",
        139 => "%8B",
        140 => "%8C",
        141 => "%8D",
        142 => "%8E",
        143 => "%8F",
        144 => "%90",
        145 => "%91",
        146 => "%92",
        147 => "%93",
        148 => "%94",
        149 => "%95",
        150 => "%96",
        151 => "%97",
        152 => "%98",
        153 => "%99",
        154 => "%9A",
        155 => "%9B",
        156 => "%9C",
        157 => "%9D",
        158 => "%9E",
        159 => "%9F",
        160 => "%A0",
        161 => "%A1",
        162 => "%A2",
        163 => "%A3",
        164 => "%A4",
        165 => "%A5",
        166 => "%A6",
        167 => "%A7",
        168 => "%A8",
        169 => "%A9",
        170 => "%AA",
        171 => "%AB",
        172 => "%AC",
        173 => "%AD",
        174 => "%AE",
        175 => "%AF",
        176 => "%B0",
        177 => "%B1",
        178 => "%B2",
        179 => "%B3",
        180 => "%B4",
        181 => "%B5",
        182 => "%B6",
        183 => "%B7",
        184 => "%B8",
        185 => "%B9",
        186 => "%BA",
        187 => "%BB",
        188 => "%BC",
        189 => "%BD",
        190 => "%BE",
        191 => "%BF",
        192 => "%C0",
        193 => "%C1",
        194 => "%C2",
        195 => "%C3",
        196 => "%C4",
        197 => "%C5",
        198 => "%C6",
        199 => "%C7",
        200 => "%C8",
        201 => "%C9",
        202 => "%CA",
        203 => "%CB",
        204 => "%CC",
        205 => "%CD",
        206 => "%CE",
        207 => "%CF",
        208 => "%D0",
        209 => "%D1",
        210 => "%D2",
        211 => "%D3",
        212 => "%D4",
        213 => "%D5",
        214 => "%D6",
        215 => "%D7",
        216 => "%D8",
        217 => "%D9",
        218 => "%DA",
        219 => "%DB",
        220 => "%DC",
        221 => "%DD",
        222 => "%DE",
        223 => "%DF",
        224 => "%E0",
        225 => "%E1",
        226 => "%E2",
        227 => "%E3",
        228 => "%E4",
        229 => "%E5",
        230 => "%E6",
        231 => "%E7",
        232 => "%E8",
        233 => "%E9",
        234 => "%EA",
        235 => "%EB",
        236 => "%EC",
        237 => "%ED",
        238 => "%EE",
        239 => "%EF",
        240 => "%F0",
        241 => "%F1",
        242 => "%F2",
        243 => "%F3",
        244 => "%F4",
        245 => "%F5",
        246 => "%F6",
        247 => "%F7",
        248 => "%F8",
        249 => "%F9",
        250 => "%FA",
        251 => "%FB",
        252 => "%FC",
        253 => "%FD",
        254 => "%FE",
        255 => "%FF",
    }
}

/// Percent-encode the given bytes with the given set.
///
/// Non-ASCII bytes and bytes in `ascii_set` are encoded.
///
/// The return type:
///
/// * Implements `Iterator<Item = &str>` and therefore has a `.collect::<String>()` method,
/// * Implements `Display` and therefore has a `.to_string()` method,
/// * Implements `Into<Cow<str>>` borrowing `input` when none of its bytes are encoded.
///
/// # Examples
///
/// ```
/// use percent_encoding::{percent_encode, NON_ALPHANUMERIC};
///
/// assert_eq!(percent_encode(b"foo bar?", NON_ALPHANUMERIC).to_string(), "foo%20bar%3F");
/// ```
#[inline]
#[ensures(result@ == percent_encode_model(input@, ascii_set@))]
pub fn percent_encode<'a>(input: &'a [u8], ascii_set: &'static AsciiSet) -> PercentEncode<'a> {
    PercentEncode {
        bytes: input,
        ascii_set,
    }
}

/// Percent-encode the UTF-8 encoding of the given string.
///
/// See [`percent_encode`] regarding the return type.
///
/// # Examples
///
/// ```
/// use percent_encoding::{utf8_percent_encode, NON_ALPHANUMERIC};
///
/// assert_eq!(utf8_percent_encode("foo bar?", NON_ALPHANUMERIC).to_string(), "foo%20bar%3F");
/// ```
#[inline]
#[ensures(result@ == percent_encode_model(input@.to_bytes(), ascii_set@))]
pub fn utf8_percent_encode<'a>(input: &'a str, ascii_set: &'static AsciiSet) -> PercentEncode<'a> {
    percent_encode(input.as_bytes(), ascii_set)
}

/// The return type of [`percent_encode`] and [`utf8_percent_encode`].
#[derive(Clone)]
pub struct PercentEncode<'a> {
    bytes: &'a [u8],
    ascii_set: &'static AsciiSet,
}

impl fmt::Debug for PercentEncode<'_> {
    #[creusot_std::prelude::trusted]
    #[ensures(creusot_std::std::fmt::formatter_extends(
        formatter.deep_model(),
        (^formatter).deep_model(),
    ))]
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("PercentEncode")
            .field("bytes", &self.bytes)
            .field("ascii_set", &self.ascii_set)
            .finish()
    }
}

impl PartialEq for PercentEncode<'_> {
    /// Equality is structural: both the remaining bytes and encode set match.
    #[ensures(result == (self.deep_model() == other.deep_model()))]
    fn eq(&self, other: &Self) -> bool {
        self.bytes == other.bytes && self.ascii_set == other.ascii_set
    }
}

impl Eq for PercentEncode<'_> {}

impl View for PercentEncode<'_> {
    type ViewTy = Seq<Int>;

    /// Exact remaining encoded byte sequence.
    #[logic]
    fn view(self) -> Self::ViewTy {
        pearlite! { percent_encode_model(self.bytes@, self.ascii_set@) }
    }
}

impl DeepModel for PercentEncode<'_> {
    type DeepModelTy = (Seq<Int>, Seq<Int>);

    /// Structural state used by the public derived equality implementation.
    /// The output-oriented [`View`] remains the exact remaining encoding.
    #[logic]
    fn deep_model(self) -> Self::DeepModelTy {
        (self.bytes.deep_model(), self.ascii_set.deep_model())
    }
}

impl Invariant for PercentEncode<'_> {
    #[logic(open)]
    fn invariant(self) -> bool {
        pearlite! {
            forall<i> 0 <= i && i < self@.len() ==> 0 <= self@[i] && self@[i] < 0x80
        }
    }
}

impl PercentEncode<'_> {
    /// Number of unconsumed input bytes in this iterator state.
    #[logic]
    pub fn remaining_input_len(self) -> Int {
        pearlite! { self.bytes@.len() }
    }

    /// Whether converting this exact iterator state into `Cow<str>` borrows.
    #[logic]
    pub fn cow_is_borrowed(self) -> bool {
        pearlite! { percent_encode_borrows(self.bytes@, self.ascii_set@) }
    }
}

impl<'a> IteratorSpec for PercentEncode<'a> {
    #[logic(open, prophetic, inline)]
    fn produces(self, visited: Seq<Self::Item>, tail: Self) -> bool {
        pearlite! { self@ == encoded_chunk_values(visited).concat(tail@) }
    }

    #[logic(open, prophetic)]
    fn completed(&mut self) -> bool {
        pearlite! { resolve(self) && (*self)@ == Seq::empty() }
    }

    #[logic(open, law)]
    #[ensures(self.produces(Seq::empty(), self))]
    fn produces_refl(self) {
        proof_assert!(encoded_chunk_values(Seq::empty()) == Seq::empty());
        proof_assert!(Seq::<Int>::empty().concat(self@) == self@)
    }

    #[logic(open, law)]
    #[requires(a.produces(ab, b))]
    #[requires(b.produces(bc, c))]
    #[ensures(a.produces(ab.concat(bc), c))]
    fn produces_trans(a: Self, ab: Seq<Self::Item>, b: Self, bc: Seq<Self::Item>, c: Self) {
        encoded_chunk_values_trans(a.view(), ab, b.view(), bc, c.view());
    }
}

/// Length of the maximal leading byte run that does not require encoding.
/// The first byte is required to belong to that run.
#[cfg(creusot)]
#[requires(bytes@.len() > 0)]
#[requires(!should_encode_model(ascii_set@, bytes@[0]))]
#[ensures(1 <= result@ && result@ <= bytes@.len())]
#[ensures(forall<i> 0 <= i && i < result@ ==>
    !should_encode_model(ascii_set@, bytes@[i]))]
#[ensures(result@ == bytes@.len() || should_encode_model(ascii_set@, bytes@[result@]))]
fn unchanged_prefix_len_verified(bytes: &[u8], ascii_set: &AsciiSet) -> usize {
    let mut index = 1usize;
    #[invariant(1 <= index@ && index@ <= bytes@.len())]
    #[invariant(forall<i> 0 <= i && i < index@ ==>
        !should_encode_model(ascii_set@, bytes@[i]))]
    while index < bytes.len() {
        let byte = bytes[index];
        if ascii_set.should_percent_encode(byte) {
            return index;
        }
        proof_assert!(!should_encode_model(ascii_set@, byte));
        index += 1;
    }
    index
}

/// Verification bridge for the standard-library fact that every ASCII byte
/// slice is valid UTF-8. The maximal-prefix search itself is body-proved.
#[cfg(creusot)]
#[creusot_std::prelude::trusted]
#[requires(forall<i> 0 <= i && i < bytes@.len() ==> bytes@[i]@ < 0x80)]
#[ensures(byte_values(result@.to_bytes()) == byte_values(bytes@))]
fn ascii_slice_as_str(bytes: &[u8]) -> &str {
    unsafe { str::from_utf8_unchecked(bytes) }
}

impl<'a> Iterator for PercentEncode<'a> {
    type Item = &'a str;

    #[ensures(match result {
        None => resolve(self) && self@ == Seq::empty(),
        Some(chunk) => self@ == byte_values(chunk@.to_bytes()).concat((^self)@),
    })]
    fn next(&mut self) -> Option<&'a str> {
        if let Some((&first_byte, remaining)) = self.bytes.split_first() {
            if self.ascii_set.should_percent_encode(first_byte) {
                self.bytes = remaining;
                Some(percent_encode_byte(first_byte))
            } else {
                #[cfg(creusot)]
                {
                    let prefix_len = unchanged_prefix_len_verified(self.bytes, self.ascii_set);
                    let old_view = snapshot!(self@);
                    let (unchanged_slice, remaining) = self.bytes.split_at(prefix_len);
                    proof_assert!(self.bytes@ == unchanged_slice@.concat(remaining@));
                    proof_assert!(forall<i> 0 <= i && i < unchanged_slice@.len() ==>
                        !should_encode_model(self.ascii_set@, unchanged_slice@[i]));
                    proof_assert! {
                        percent_encode_unencoded_concat(
                            unchanged_slice@,
                            remaining@,
                            self.ascii_set@,
                        );
                        *old_view == byte_values(unchanged_slice@).concat(
                            percent_encode_model(remaining@, self.ascii_set@),
                        )
                    };
                    self.bytes = remaining;
                    return Some(ascii_slice_as_str(unchanged_slice));
                }

                #[cfg(not(creusot))]
                {
                    // The unsafe blocks here are appropriate because the bytes are
                    // confirmed as a subset of UTF-8 in should_percent_encode.
                    let mut i = 0usize;
                    while i < remaining.len() {
                        let byte = remaining[i];
                        if self.ascii_set.should_percent_encode(byte) {
                            // 1 for first_byte + i for previous iterations of this loop
                            let (unchanged_slice, remaining) = self.bytes.split_at(1 + i);
                            self.bytes = remaining;
                            return Some(unsafe { str::from_utf8_unchecked(unchanged_slice) });
                        }
                        i += 1;
                    }
                    let unchanged_slice = self.bytes;
                    self.bytes = &[];
                    Some(unsafe { str::from_utf8_unchecked(unchanged_slice) })
                }
            }
        } else {
            None
        }
    }

    #[ensures(result.0@ == if self.remaining_input_len() == 0 { 0 } else { 1 })]
    #[ensures(result.1.deep_model() == Some(self.remaining_input_len()))]
    fn size_hint(&self) -> (usize, Option<usize>) {
        if self.bytes.is_empty() {
            (0, Some(0))
        } else {
            (1, Some(self.bytes.len()))
        }
    }
}

impl fmt::Display for PercentEncode<'_> {
    #[creusot_std::prelude::trusted]
    #[ensures(exists<i> 0 <= i && i <= self@.len()
        && (^formatter).deep_model() == formatter.deep_model().concat(self@.subsequence(0, i))
        && match result { Ok(_) => i == self@.len(), Err(_) => true })]
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        for c in (*self).clone() {
            formatter.write_str(c)?
        }
        Ok(())
    }
}

#[cfg(feature = "alloc")]
impl<'a> From<PercentEncode<'a>> for Cow<'a, str> {
    #[creusot_std::prelude::trusted]
    #[ensures(byte_values(result@.to_bytes()) == iter@)]
    #[ensures(result.is_borrowed() == iter.cow_is_borrowed())]
    fn from(mut iter: PercentEncode<'a>) -> Self {
        match iter.next() {
            None => "".into(),
            Some(first) => match iter.next() {
                None => first.into(),
                Some(second) => {
                    let mut string = first.to_owned();
                    string.push_str(second);
                    for chunk in iter {
                        string.push_str(chunk);
                    }
                    string.into()
                }
            },
        }
    }
}

/// Percent-decode the given string.
///
/// <https://url.spec.whatwg.org/#string-percent-decode>
///
/// See [`percent_decode`] regarding the return type.
#[inline]
#[ensures(result@ == percent_decode_model(input@.to_bytes()))]
pub fn percent_decode_str(input: &str) -> PercentDecode<'_> {
    percent_decode(input.as_bytes())
}

/// Percent-decode the given bytes.
///
/// <https://url.spec.whatwg.org/#percent-decode>
///
/// Any sequence of `%` followed by two hexadecimal digits is decoded.
/// The return type:
///
/// * Implements `Into<Cow<u8>>` borrowing `input` when it contains no percent-encoded sequence,
/// * Implements `Iterator<Item = u8>` and therefore has a `.collect::<Vec<u8>>()` method,
/// * Has `decode_utf8()` and `decode_utf8_lossy()` methods.
///
/// # Examples
///
/// ```
/// use percent_encoding::percent_decode;
///
/// assert_eq!(percent_decode(b"foo%20bar%3f").decode_utf8().unwrap(), "foo bar?");
/// ```
#[inline]
#[ensures(result@ == percent_decode_model(input@))]
pub fn percent_decode(input: &[u8]) -> PercentDecode<'_> {
    PercentDecode { bytes: input }
}

/// The return type of [`percent_decode`].
#[derive(Clone)]
pub struct PercentDecode<'a> {
    bytes: &'a [u8],
}

impl fmt::Debug for PercentDecode<'_> {
    #[creusot_std::prelude::trusted]
    #[ensures(creusot_std::std::fmt::formatter_extends(
        formatter.deep_model(),
        (^formatter).deep_model(),
    ))]
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("PercentDecode")
            .field("bytes", &self.bytes)
            .finish()
    }
}

impl View for PercentDecode<'_> {
    type ViewTy = Seq<Int>;

    /// Exact remaining decoded byte sequence.
    #[logic]
    fn view(self) -> Self::ViewTy {
        pearlite! { percent_decode_model(self.bytes@) }
    }
}

impl DeepModel for PercentDecode<'_> {
    type DeepModelTy = Seq<Int>;

    #[logic]
    fn deep_model(self) -> Self::DeepModelTy {
        pearlite! { self@ }
    }
}

impl Invariant for PercentDecode<'_> {
    #[logic(open)]
    fn invariant(self) -> bool {
        pearlite! {
            forall<i> 0 <= i && i < self@.len() ==> 0 <= self@[i] && self@[i] <= 0xff
        }
    }
}

impl PercentDecode<'_> {
    /// Number of unconsumed input bytes in this iterator state.
    #[logic]
    pub fn remaining_input_len(self) -> Int {
        pearlite! { self.bytes@.len() }
    }

    /// Whether converting this exact iterator state into `Cow<[u8]>` borrows.
    #[logic]
    pub fn cow_is_borrowed(self) -> bool {
        pearlite! { !has_percent_escape(self.bytes@) }
    }
}

impl<'a> IteratorSpec for PercentDecode<'a> {
    #[logic(open, prophetic, inline)]
    fn produces(self, visited: Seq<Self::Item>, tail: Self) -> bool {
        pearlite! { self@ == byte_values(visited).concat(tail@) }
    }

    #[logic(open, prophetic)]
    fn completed(&mut self) -> bool {
        pearlite! { resolve(self) && (*self)@ == Seq::empty() }
    }

    #[logic(open, law)]
    #[ensures(self.produces(Seq::empty(), self))]
    fn produces_refl(self) {
        proof_assert!(byte_values(Seq::empty()) == Seq::empty());
        proof_assert!(Seq::<Int>::empty().concat(self@) == self@)
    }

    #[logic(open, law)]
    #[requires(a.produces(ab, b))]
    #[requires(b.produces(bc, c))]
    #[ensures(a.produces(ab.concat(bc), c))]
    fn produces_trans(a: Self, ab: Seq<Self::Item>, b: Self, bc: Seq<Self::Item>, c: Self) {
        byte_values_concat(ab, bc);
        proof_assert!(a@ == byte_values(ab).concat(b@));
        proof_assert!(b@ == byte_values(bc).concat(c@));
        proof_assert!(a@ == byte_values(ab).concat(byte_values(bc).concat(c@)));
        proof_assert!(byte_values(ab).concat(byte_values(bc).concat(c@))
            == byte_values(ab).concat(byte_values(bc)).concat(c@));
        proof_assert!(a@ == byte_values(ab.concat(bc)).concat(c@));
        proof_assert!(a.produces(ab.concat(bc), c))
    }
}

#[ensures(match result {
    Some(value) => is_hex_digit_model(byte) && value@ == hex_value_model(byte),
    None => !is_hex_digit_model(byte),
})]
fn decode_hex_digit(byte: u8) -> Option<u8> {
    match byte {
        b'0'..=b'9' => Some(byte - b'0'),
        b'A'..=b'F' => Some(byte - b'A' + 10),
        b'a'..=b'f' => Some(byte - b'a' + 10),
        _ => None,
    }
}

#[cfg(not(creusot))]
fn after_percent_sign(iter: &mut slice::Iter<'_, u8>) -> Option<u8> {
    let mut cloned_iter = iter.clone();
    let h = decode_hex_digit(*cloned_iter.next()?)?;
    let l = decode_hex_digit(*cloned_iter.next()?)?;
    *iter = cloned_iter;
    Some(h * 0x10 + l)
}

impl Iterator for PercentDecode<'_> {
    type Item = u8;

    #[ensures(match result {
        None => resolve(self) && self@ == Seq::empty(),
        Some(byte) => self@ == Seq::singleton(byte@).concat((^self)@),
    })]
    fn next(&mut self) -> Option<u8> {
        let (&byte, remaining) = self.bytes.split_first()?;
        self.bytes = remaining;
        if byte == b'%' && remaining.len() >= 2 {
            if let (Some(high), Some(low)) = (
                decode_hex_digit(remaining[0]),
                decode_hex_digit(remaining[1]),
            ) {
                self.bytes = &remaining[2..];
                return Some(high * 0x10 + low);
            }
        }
        Some(byte)
    }

    #[ensures(result.0@ == self.remaining_input_len() / 3
        + if self.remaining_input_len() % 3 == 0 { 0 } else { 1 })]
    #[ensures(result.1.deep_model() == Some(self.remaining_input_len()))]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let bytes = self.bytes.len();
        (bytes / 3 + if bytes % 3 == 0 { 0 } else { 1 }, Some(bytes))
    }
}

#[cfg(feature = "alloc")]
impl<'a> From<PercentDecode<'a>> for Cow<'a, [u8]> {
    #[ensures(byte_values(result@) == iter@)]
    #[ensures(result.is_borrowed() == iter.cow_is_borrowed())]
    fn from(iter: PercentDecode<'a>) -> Self {
        match iter.if_any() {
            Some(vec) => Cow::Owned(vec),
            None => Cow::Borrowed(iter.bytes),
        }
    }
}

impl<'a> PercentDecode<'a> {
    /// If the percent-decoding is different from the input, return it as a new bytes vector.
    #[cfg(feature = "alloc")]
    #[cfg(not(creusot))]
    fn if_any(&self) -> Option<Vec<u8>> {
        let mut bytes_iter = self.bytes.iter();
        while bytes_iter.any(|&b| b == b'%') {
            if let Some(decoded_byte) = after_percent_sign(&mut bytes_iter) {
                let initial_bytes = self.bytes;
                let unchanged_bytes_len = initial_bytes.len() - bytes_iter.len() - 3;
                let mut decoded = initial_bytes[..unchanged_bytes_len].to_owned();
                decoded.push(decoded_byte);
                decoded.extend(PercentDecode {
                    bytes: bytes_iter.as_slice(),
                });
                return Some(decoded);
            }
        }
        // Nothing to decode
        None
    }

    #[cfg(all(feature = "alloc", creusot))]
    #[creusot_std::prelude::trusted]
    #[ensures(match result {
        Some(value) => has_percent_escape(self.bytes@) && byte_values(value@) == self@,
        None => !has_percent_escape(self.bytes@) && byte_values(self.bytes@) == self@,
    })]
    fn if_any(&self) -> Option<Vec<u8>> {
        panic!("verification-only allocation adapter")
    }

    /// Decode the result of percent-decoding as UTF-8.
    ///
    /// This is return `Err` when the percent-decoded bytes are not well-formed in UTF-8.
    #[cfg(feature = "alloc")]
    #[creusot_std::prelude::trusted]
    #[ensures(match result {
        Ok(value) => byte_values(value@.to_bytes()) == self@,
        Err(error) => exists<decoded: Seq<u8>> byte_values(decoded) == self@
            && utf8_error_matches(decoded, error.valid_up_to_logic(), error.error_len_logic()),
    })]
    pub fn decode_utf8(self) -> Result<Cow<'a, str>, str::Utf8Error> {
        match self.clone().into() {
            Cow::Borrowed(bytes) => match str::from_utf8(bytes) {
                Ok(s) => Ok(s.into()),
                Err(e) => Err(e),
            },
            Cow::Owned(bytes) => match String::from_utf8(bytes) {
                Ok(s) => Ok(s.into()),
                Err(e) => Err(e.utf8_error()),
            },
        }
    }

    /// Decode the result of percent-decoding as UTF-8, lossily.
    ///
    /// Invalid UTF-8 percent-encoded byte sequences will be replaced � U+FFFD,
    /// the replacement character.
    #[cfg(feature = "alloc")]
    #[creusot_std::prelude::trusted]
    #[ensures(exists<decoded: Seq<u8>> byte_values(decoded) == self@
        && result@ == utf8_lossy_model(decoded))]
    pub fn decode_utf8_lossy(self) -> Cow<'a, str> {
        decode_utf8_lossy(self.clone().into())
    }
}

// std::ptr::addr_eq was stabilized in rust 1.76. Once we upgrade
// the MSRV we can remove this lint override.
#[cfg(feature = "alloc")]
#[allow(ambiguous_wide_pointer_comparisons)]
#[creusot_std::prelude::trusted]
#[ensures(result@ == utf8_lossy_model(input@))]
#[ensures(result.is_borrowed() == (input.is_borrowed() && valid_utf8(input@)))]
fn decode_utf8_lossy(input: Cow<'_, [u8]>) -> Cow<'_, str> {
    // Note: This function is duplicated in `form_urlencoded/src/query_encoding.rs`.
    match input {
        Cow::Borrowed(bytes) => String::from_utf8_lossy(bytes),
        Cow::Owned(bytes) => {
            match String::from_utf8_lossy(&bytes) {
                Cow::Borrowed(utf8) => {
                    // If from_utf8_lossy returns a Cow::Borrowed, then we can
                    // be sure our original bytes were valid UTF-8. This is because
                    // if the bytes were invalid UTF-8 from_utf8_lossy would have
                    // to allocate a new owned string to back the Cow so it could
                    // replace invalid bytes with a placeholder.

                    // First we do a debug_assert to confirm our description above.
                    let raw_utf8: *const [u8] = utf8.as_bytes();
                    #[cfg(not(creusot))]
                    debug_assert!(core::ptr::eq(raw_utf8, &*bytes));

                    #[cfg(creusot)]
                    let _ = raw_utf8;

                    // Given we know the original input bytes are valid UTF-8,
                    // and we have ownership of those bytes, we re-use them and
                    // return a Cow::Owned here.
                    Cow::Owned(unsafe { String::from_utf8_unchecked(bytes) })
                }
                Cow::Owned(s) => Cow::Owned(s),
            }
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn percent_encode_byte() {
        for i in 0..=0xFF {
            let encoded = super::percent_encode_byte(i);
            assert_eq!(encoded, alloc::format!("%{:02X}", i));
        }
    }

    #[test]
    fn percent_encode_accepts_ascii_set_ref() {
        let encoded = percent_encode(b"foo bar?", &AsciiSet::EMPTY);
        assert_eq!(encoded.collect::<String>(), "foo bar?");
    }

    #[test]
    fn percent_encode_collect() {
        let encoded = percent_encode(b"foo bar?", NON_ALPHANUMERIC);
        assert_eq!(encoded.collect::<String>(), String::from("foo%20bar%3F"));

        let encoded = percent_encode(b"\x00\x01\x02\x03", CONTROLS);
        assert_eq!(encoded.collect::<String>(), String::from("%00%01%02%03"));
    }

    #[test]
    fn percent_encode_chunks_match_the_iterator_contract() {
        let mut encoded = percent_encode(b"ab c?", NON_ALPHANUMERIC);
        assert_eq!(encoded.next(), Some("ab"));
        assert_eq!(encoded.next(), Some("%20"));
        assert_eq!(encoded.next(), Some("c"));
        assert_eq!(encoded.next(), Some("%3F"));
        assert_eq!(encoded.next(), None);
    }

    #[test]
    fn percent_encode_display() {
        let encoded = percent_encode(b"foo bar?", NON_ALPHANUMERIC);
        assert_eq!(alloc::format!("{}", encoded), "foo%20bar%3F");
    }

    #[test]
    fn public_debug_output_is_preserved() {
        assert_eq!(
            alloc::format!("{:?}", AsciiSet::EMPTY),
            "AsciiSet { mask: [0, 0, 0, 0] }"
        );
        assert_eq!(
            alloc::format!("{:?}", percent_encode(b"ab", &AsciiSet::EMPTY)),
            "PercentEncode { bytes: [97, 98], ascii_set: AsciiSet { mask: [0, 0, 0, 0] } }"
        );
        assert_eq!(
            alloc::format!("{:?}", super::percent_decode(b"ab")),
            "PercentDecode { bytes: [97, 98] }"
        );
    }

    #[test]
    fn percent_encode_cow() {
        let encoded = percent_encode(b"foo bar?", NON_ALPHANUMERIC);
        assert_eq!(Cow::from(encoded), "foo%20bar%3F");
    }

    #[test]
    fn utf8_percent_encode_accepts_ascii_set_ref() {
        let encoded = super::utf8_percent_encode("foo bar?", &AsciiSet::EMPTY);
        assert_eq!(encoded.collect::<String>(), "foo bar?");
    }

    #[test]
    fn utf8_percent_encode() {
        assert_eq!(
            super::utf8_percent_encode("foo bar?", NON_ALPHANUMERIC),
            percent_encode(b"foo bar?", NON_ALPHANUMERIC)
        );
    }

    #[test]
    fn percent_decode() {
        assert_eq!(
            super::percent_decode(b"foo%20bar%3f")
                .decode_utf8()
                .unwrap(),
            "foo bar?"
        );
    }

    #[test]
    fn percent_decode_str() {
        assert_eq!(
            super::percent_decode_str("foo%20bar%3f")
                .decode_utf8()
                .unwrap(),
            "foo bar?"
        );
    }

    #[test]
    fn percent_decode_collect() {
        let decoded = super::percent_decode(b"foo%20bar%3f");
        assert_eq!(decoded.collect::<Vec<u8>>(), b"foo bar?");
    }

    #[test]
    fn percent_decode_cow() {
        let decoded = super::percent_decode(b"foo%20bar%3f");
        assert_eq!(Cow::from(decoded), Cow::Owned::<[u8]>(b"foo bar?".to_vec()));

        let decoded = super::percent_decode(b"foo bar?");
        assert_eq!(Cow::from(decoded), Cow::Borrowed(b"foo bar?"));
    }

    #[test]
    fn percent_decode_invalid_utf8() {
        // Invalid UTF-8 sequence
        let decoded = super::percent_decode(b"%00%9F%92%96")
            .decode_utf8()
            .unwrap_err();
        assert_eq!(decoded.valid_up_to(), 1);
        assert_eq!(decoded.error_len(), Some(1));
    }

    #[test]
    fn percent_decode_utf8_lossy() {
        assert_eq!(
            super::percent_decode(b"%F0%9F%92%96").decode_utf8_lossy(),
            "💖"
        );
    }

    #[test]
    fn percent_decode_utf8_lossy_invalid_utf8() {
        assert_eq!(
            super::percent_decode(b"%00%9F%92%96").decode_utf8_lossy(),
            "\u{0}���"
        );
    }
}
