//! `FixedBitSet` is a simple fixed size set of bits.

#![no_std]
#![allow(unexpected_cfgs)]

extern crate alloc;
extern crate creusot_std;

#[cfg(not(creusot))]
mod runtime;
#[cfg(not(creusot))]
pub use runtime::*;

// The upstream representation owns an aligned SIMD allocation through a raw
// pointer. Creusot cannot currently translate that ownership code, so the
// verification build uses an element-wise state machine with the same core
// public transitions. See PROVENANCE.md for the boundary and removal condition.
#[cfg(creusot)]
mod verification;
#[cfg(creusot)]
pub use verification::*;
