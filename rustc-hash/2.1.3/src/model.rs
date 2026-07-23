//! Mathematical model used by the `rustc-hash` contracts.
//!
//! The byte hash is split into little-endian decoding, one 16-byte bulk
//! transition, a recursive fold over complete bulk blocks, and finalization.
//! This keeps the public `Hasher::write` contract independent of the optimized
//! slice implementation.

#[allow(unused_imports)]
use creusot_std::prelude::{ensures, logic, pearlite, requires, variant, Int, Seq};

pub const MODEL_SEED1: u64 = 0x243f6a8885a308d3;
pub const MODEL_SEED2: u64 = 0x13198a2e03707344;
pub const MODEL_PREVENT_ZERO: u64 = 0xa4093822299f31d0;
#[cfg(target_pointer_width = "64")]
pub const MODEL_K: usize = 0xf1357aea2e62a9c5;
#[cfg(target_pointer_width = "32")]
pub const MODEL_K: usize = 0x93d765dd;

/// One machine-word polynomial-hash transition.
#[logic(open)]
pub fn fx_step(hash: usize, value: usize) -> usize {
    pearlite! { (hash + value) * MODEL_K }
}

/// The platform-specific sequence of transitions for a `u64` input.
#[cfg(target_pointer_width = "64")]
#[logic(open)]
pub fn fx_write_u64(hash: usize, value: u64) -> usize {
    pearlite! { fx_step(hash, value as usize) }
}

/// The platform-specific sequence of transitions for a `u64` input.
#[cfg(target_pointer_width = "32")]
#[logic(open)]
pub fn fx_write_u64(hash: usize, value: u64) -> usize {
    pearlite! {
        fx_step(fx_step(hash, value as usize), (value >> 32u8) as usize)
    }
}

/// The platform-specific sequence of transitions for a `u128` input.
#[cfg(target_pointer_width = "64")]
#[logic(open)]
pub fn fx_write_u128(hash: usize, value: u128) -> usize {
    pearlite! {
        fx_step(fx_step(hash, value as usize), (value >> 64u8) as usize)
    }
}

/// The platform-specific sequence of transitions for a `u128` input.
#[cfg(target_pointer_width = "32")]
#[logic(open)]
pub fn fx_write_u128(hash: usize, value: u128) -> usize {
    pearlite! {
        fx_step(
            fx_step(
                fx_step(fx_step(hash, value as usize), (value >> 32u8) as usize),
                (value >> 64u8) as usize,
            ),
            (value >> 96u8) as usize,
        )
    }
}

/// The rotation used by `Hasher::finish` on the selected pointer width.
#[cfg(target_pointer_width = "64")]
#[logic(open)]
pub fn finish_model(hash: usize) -> u64 {
    pearlite! { ((hash << 26u8) | (hash >> 38u8)) as u64 }
}

/// The rotation used by `Hasher::finish` on the selected pointer width.
#[cfg(target_pointer_width = "32")]
#[logic(open)]
pub fn finish_model(hash: usize) -> u64 {
    pearlite! { ((hash << 15u8) | (hash >> 17u8)) as u64 }
}

/// Decode eight bytes starting at `start` in little-endian order.
#[logic(open)]
#[requires(0 <= start && start + 8 <= bytes.len())]
pub fn read_u64_le(bytes: Seq<u8>, start: Int) -> u64 {
    pearlite! {
        (bytes[start] as u64)
            | ((bytes[start + 1] as u64) << 8u8)
            | ((bytes[start + 2] as u64) << 16u8)
            | ((bytes[start + 3] as u64) << 24u8)
            | ((bytes[start + 4] as u64) << 32u8)
            | ((bytes[start + 5] as u64) << 40u8)
            | ((bytes[start + 6] as u64) << 48u8)
            | ((bytes[start + 7] as u64) << 56u8)
    }
}

/// Decode four bytes starting at `start` in little-endian order.
#[logic(open)]
#[requires(0 <= start && start + 4 <= bytes.len())]
pub fn read_u32_le(bytes: Seq<u8>, start: Int) -> u32 {
    pearlite! {
        (bytes[start] as u32)
            | ((bytes[start + 1] as u32) << 8u8)
            | ((bytes[start + 2] as u32) << 16u8)
            | ((bytes[start + 3] as u32) << 24u8)
    }
}

/// The multiply-and-fold primitive used on targets with widening multiply.
#[cfg(any(
    all(
        target_pointer_width = "64",
        not(any(target_arch = "sparc64", target_arch = "wasm64")),
    ),
    target_arch = "aarch64",
    target_arch = "x86_64",
    all(target_family = "wasm", target_feature = "wide-arithmetic"),
))]
#[logic(open)]
pub fn multiply_mix_model(x: u64, y: u64) -> u64 {
    pearlite! {
        let full = (x as u128) * (y as u128);
        (full as u64) ^ ((full >> 64u8) as u64)
    }
}

/// The decomposed multiply-and-fold primitive used without widening multiply.
#[cfg(not(any(
    all(
        target_pointer_width = "64",
        not(any(target_arch = "sparc64", target_arch = "wasm64")),
    ),
    target_arch = "aarch64",
    target_arch = "x86_64",
    all(target_family = "wasm", target_feature = "wide-arithmetic"),
)))]
#[logic(open)]
pub fn multiply_mix_model(x: u64, y: u64) -> u64 {
    pearlite! {
        let lx = x as u32;
        let ly = y as u32;
        let hx = (x >> 32u8) as u32;
        let hy = (y >> 32u8) as u32;
        let afull = (lx as u64) * (hy as u64);
        let bfull = (hx as u64) * (ly as u64);
        afull ^ ((bfull >> 32u8) | (bfull << 32u8))
    }
}

/// One complete 16-byte bulk transition.
#[logic(open)]
#[requires(0 <= start && start + 16 <= bytes.len())]
pub fn bulk_step(bytes: Seq<u8>, start: Int, state: (u64, u64)) -> (u64, u64) {
    pearlite! {
        let x = read_u64_le(bytes, start);
        let y = read_u64_le(bytes, start + 8);
        (
            state.1,
            multiply_mix_model(state.0 ^ x, MODEL_PREVENT_ZERO ^ y),
        )
    }
}

/// Fold the first `count` complete 16-byte blocks.
#[logic]
#[requires(0 <= count && count * 16 <= bytes.len())]
#[variant(count)]
pub fn bulk_fold(bytes: Seq<u8>, count: Int, state: (u64, u64)) -> (u64, u64) {
    if count == 0 {
        state
    } else {
        pearlite! {
            bulk_step(
                bytes,
                (count - 1) * 16,
                bulk_fold(bytes, count - 1, state),
            )
        }
    }
}

/// The empty block prefix leaves the state unchanged.
#[logic]
#[ensures(bulk_fold(bytes, 0, state) == state)]
pub fn bulk_fold_zero(bytes: Seq<u8>, state: (u64, u64)) {}

/// Unfold one additional complete block in the forward direction used by the
/// verified runtime loop.
#[logic]
#[requires(0 <= count && (count + 1) * 16 <= bytes.len())]
#[ensures(bulk_fold(bytes, count + 1, state)
    == bulk_step(bytes, count * 16, bulk_fold(bytes, count, state)))]
pub fn bulk_fold_succ(bytes: Seq<u8>, count: Int, state: (u64, u64)) {}

/// Exact byte-compression model used by `Hasher::write`.
#[logic(open)]
#[requires(len_word@ == bytes.len())]
pub fn hash_bytes_model(bytes: Seq<u8>, len_word: u64) -> u64 {
    pearlite! {
        if bytes.len() <= 16 {
            let state = if bytes.len() >= 8 {
                (
                    MODEL_SEED1 ^ read_u64_le(bytes, 0),
                    MODEL_SEED2 ^ read_u64_le(bytes, bytes.len() - 8),
                )
            } else if bytes.len() >= 4 {
                (
                    MODEL_SEED1 ^ (read_u32_le(bytes, 0) as u64),
                    MODEL_SEED2 ^ (read_u32_le(bytes, bytes.len() - 4) as u64),
                )
            } else if bytes.len() > 0 {
                (
                    MODEL_SEED1 ^ (bytes[0] as u64),
                    MODEL_SEED2
                        ^ (((bytes[bytes.len() - 1] as u64) << 8u8)
                            | (bytes[bytes.len() / 2] as u64)),
                )
            } else {
                (MODEL_SEED1, MODEL_SEED2)
            };
            multiply_mix_model(state.0, state.1) ^ len_word
        } else {
            let state = bulk_fold(
                bytes,
                (bytes.len() - 1) / 16,
                (MODEL_SEED1, MODEL_SEED2),
            );
            multiply_mix_model(
                state.0 ^ read_u64_le(bytes, bytes.len() - 16),
                state.1 ^ read_u64_le(bytes, bytes.len() - 8),
            ) ^ len_word
        }
    }
}
