//! Concurrent queues.
//!
//! This crate provides concurrent queues that can be shared among threads:
//!
//! * [`ArrayQueue`], a bounded MPMC queue that allocates a fixed-capacity buffer on construction.
//! * [`SegQueue`], an unbounded MPMC queue that allocates small buffers, segments, on demand.

#![no_std]
#![doc(test(
    no_crate_inject,
    attr(
        deny(warnings, rust_2018_idioms),
        allow(dead_code, unused_assignments, unused_variables)
    )
))]
#![warn(
    missing_docs,
    missing_debug_implementations,
    rust_2018_idioms,
    unreachable_pub
)]
#![allow(unexpected_cfgs)]

#[allow(unused_extern_crates)]
extern crate creusot_std;

#[cfg(all(feature = "alloc", target_has_atomic = "ptr"))]
extern crate alloc;
#[cfg(feature = "std")]
extern crate std;

#[cfg(all(feature = "alloc", target_has_atomic = "ptr", not(creusot)))]
mod array_queue;
#[cfg(all(feature = "alloc", target_has_atomic = "ptr", not(creusot)))]
mod seg_queue;

#[cfg(all(feature = "alloc", target_has_atomic = "ptr", not(creusot)))]
pub use crate::{array_queue::ArrayQueue, seg_queue::SegQueue};

// Creusot cannot currently translate the lock-free implementation's raw
// pointers, UnsafeCell slots, and compare-exchange loops. Verify the exact FIFO
// state transitions available under exclusive access in a separate projection.
// The runtime build above remains the unmodified upstream implementation.
#[cfg(all(feature = "alloc", creusot))]
mod verification;
#[cfg(all(feature = "alloc", creusot))]
pub use crate::verification::{ArrayQueue, SegQueue};
