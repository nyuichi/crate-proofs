//! `fugit` provides a comprehensive library of [`Duration`] and [`Instant`] for the handling of
//! time in embedded systems. The library is specifically designed to maximize const-ification
//! which allows for most comparisons and changes of time-base to be made at compile time, rather
//! than run time.
//!
//! The library is aimed at ease-of-use and performance first.
//!
//! ```
//! use fugit::{Duration, ExtU32};
//!
//! // Efficient short-hands (`.millis()`, ...)
//! let d = Duration::<u32, 1, 1_000>::from_ticks(111);
//!
//! let sum1 = d + 300.millis();
//! //             ^^^ Compile time move of base, only a sum is needed and no change of base
//!
//!
//! // -----------------------
//!
//! // Best effort for fixed types
//! fn bar(d1: Duration<u32, 1, 1_000>, d2: Duration<u32, 1, 1_000_000>) {
//!     let sum = d1 + d2.convert();
//!     //        ^^^^^^^ Run time move of base, will use a `mul` and `div` instruction (Cortex-M3+) to
//!     //                perform the move of base.
//!     //                The `.convert()` explicitly signals the move of base.
//!
//!     let ops = d1 > d2;
//!     //        ^^^^^^^ Run time comparison of different base, will use 2 `mul` instructions
//!     //                (Cortex-M3+) to perform the comparison.
//! }
//!
//! fn baz(d1: Duration<u64, 1, 1_000>, d2: Duration<u64, 1, 1_000_000>) {
//!     let sum = d1 + d2.convert();
//!     //        ^^^^^^^ Run time move of base, will use a `mul` insruction and `div`
//!     //                soft-impl (Cortex-M3+) to perform the move of base.
//!     //                The `.convert()` explicitly signals the move of base.
//!
//!     let ops = d1 > d2;
//!     //        ^^^^^^^ Run time comparison of different base, will use 4 `mul` instructions
//!     //                (Cortex-M3+) to perform the comparison.
//! }
//! ```

#![cfg_attr(not(test), no_std)]
#![deny(missing_docs)]

extern crate creusot_std;

#[allow(unused_imports)]
use creusot_std::prelude::{
    ensures, invariant, logic, pearlite, requires, trusted, Int, Invariant, View,
};

/// Mathematical floor conversion between two tick bases.
///
/// This is the common model used by `Duration` and `Rate` conversion contracts.
#[logic(open)]
pub fn scale_floor(
    value: Int,
    source_nom: u64,
    source_denom: u64,
    target_nom: u64,
    target_denom: u64,
) -> Int {
    if source_denom == 0u64 || target_nom == 0u64 {
        0
    } else {
        pearlite! { value * source_nom@ * target_denom@ / (source_denom@ * target_nom@) }
    }
}

/// Mathematical ceiling conversion between two tick bases.
#[logic(open)]
pub fn scale_ceil(
    value: Int,
    source_nom: u64,
    source_denom: u64,
    target_nom: u64,
    target_denom: u64,
) -> Int {
    if source_denom == 0u64 || target_nom == 0u64 {
        0
    } else {
        pearlite! {
            let numerator = value * source_nom@ * target_denom@;
            let denominator = source_denom@ * target_nom@;
            (numerator + denominator - 1) / denominator
        }
    }
}

/// Mathematical nearest-integer conversion between two tick bases, with ties
/// rounded upward.
#[logic(open)]
pub fn scale_nearest(
    value: Int,
    source_nom: u64,
    source_denom: u64,
    target_nom: u64,
    target_denom: u64,
) -> Int {
    if source_denom == 0u64 || target_nom == 0u64 {
        0
    } else {
        pearlite! {
            let numerator = value * source_nom@ * target_denom@;
            let denominator = source_denom@ * target_nom@;
            let quotient = numerator / denominator;
            let remainder = numerator % denominator;
            if remainder >= denominator - remainder { quotient + 1 } else { quotient }
        }
    }
}

/// One side of a cross-multiplied physical quantity comparison.
#[logic(open)]
pub fn cross_product(value: Int, nom: u64, other_denom: u64) -> Int {
    pearlite! { value * nom@ * other_denom@ }
}

/// Mathematical reciprocal conversion between a period and a rate.
#[logic(open)]
pub fn reciprocal_scale(
    value: Int,
    period_nom: u64,
    period_denom: u64,
    rate_nom: u64,
    rate_denom: u64,
) -> Int {
    if value == 0 || period_nom == 0u64 || rate_nom == 0u64 {
        0
    } else {
        pearlite! {
            period_denom@ * rate_denom@ / (value * period_nom@ * rate_nom@)
        }
    }
}

mod aliases;
mod duration;
mod helpers;
mod instant;
mod rate;

pub use aliases::*;
pub use duration::{Duration, ExtU32, ExtU32Ceil, ExtU64, ExtU64Ceil};
pub use instant::Instant;
pub use rate::{ExtU32 as RateExtU32, ExtU64 as RateExtU64, Rate};

#[cfg(test)]
mod test_duration;

#[cfg(test)]
mod test_instant;

#[cfg(test)]
mod test_rate;
