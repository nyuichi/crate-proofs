use crate::*;

#[allow(unused_imports)]
use creusot_std::prelude::{ensures, trusted};

/// Trusted eight-round bitvector boundary. TODO: prove the loop against
/// `crc8_rounds_model` using its counter as the sole progress measure.
#[trusted]
#[ensures(result == crc8_byte_model(poly, reflect, value))]
pub(crate) const fn crc8(poly: u8, reflect: bool, mut value: u8) -> u8 {
    let mut i = 0;
    if reflect {
        while i < 8 {
            value = (value >> 1) ^ ((value & 1) * poly);
            i += 1;
        }
    } else {
        while i < 8 {
            value = (value << 1) ^ (((value >> 7) & 1) * poly);
            i += 1;
        }
    }
    value
}

/// Trusted eight-round bitvector boundary. TODO: prove the loop against
/// `crc16_rounds_model` using its counter as the sole progress measure.
#[trusted]
#[ensures(result == crc16_byte_model(poly, reflect, value))]
pub(crate) const fn crc16(poly: u16, reflect: bool, mut value: u16) -> u16 {
    if reflect {
        let mut i = 0;
        while i < 8 {
            value = (value >> 1) ^ ((value & 1) * poly);
            i += 1;
        }
    } else {
        value <<= 8;

        let mut i = 0;
        while i < 8 {
            value = (value << 1) ^ (((value >> 15) & 1) * poly);
            i += 1;
        }
    }
    value
}

/// Trusted eight-round bitvector boundary. TODO: prove the loop against
/// `crc32_rounds_model` using its counter as the sole progress measure.
#[trusted]
#[ensures(result == crc32_byte_model(poly, reflect, value))]
pub(crate) const fn crc32(poly: u32, reflect: bool, mut value: u32) -> u32 {
    if reflect {
        let mut i = 0;
        while i < 8 {
            value = (value >> 1) ^ ((value & 1) * poly);
            i += 1;
        }
    } else {
        value <<= 24;

        let mut i = 0;
        while i < 8 {
            value = (value << 1) ^ (((value >> 31) & 1) * poly);
            i += 1;
        }
    }
    value
}

/// Trusted eight-round bitvector boundary. TODO: prove the loop against
/// `crc64_rounds_model` using its counter as the sole progress measure.
#[trusted]
#[ensures(result == crc64_byte_model(poly, reflect, value))]
pub(crate) const fn crc64(poly: u64, reflect: bool, mut value: u64) -> u64 {
    if reflect {
        let mut i = 0;
        while i < 8 {
            value = (value >> 1) ^ ((value & 1) * poly);
            i += 1;
        }
    } else {
        value <<= 56;

        let mut i = 0;
        while i < 8 {
            value = (value << 1) ^ (((value >> 63) & 1) * poly);
            i += 1;
        }
    }
    value
}

/// Trusted eight-round bitvector boundary. TODO: prove the loop against
/// `crc128_rounds_model` using its counter as the sole progress measure.
#[trusted]
#[ensures(result == crc128_byte_model(poly, reflect, value))]
pub(crate) const fn crc128(poly: u128, reflect: bool, mut value: u128) -> u128 {
    if reflect {
        let mut i = 0;
        while i < 8 {
            value = (value >> 1) ^ ((value & 1) * poly);
            i += 1;
        }
    } else {
        value <<= 120;

        let mut i = 0;
        while i < 8 {
            value = (value << 1) ^ (((value >> 127) & 1) * poly);
            i += 1;
        }
    }
    value
}
