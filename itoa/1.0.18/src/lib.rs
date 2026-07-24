//! Fast conversion of integer primitives to decimal strings.
//!
//! ```
//! let mut buffer = itoa::Buffer::new();
//! assert_eq!(buffer.format(128u64), "128");
//! ```

#![doc(html_root_url = "https://docs.rs/itoa/1.0.18")]
#![no_std]
#![allow(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::expl_impl_clone_on_copy,
    clippy::identity_op,
    clippy::items_after_statements,
    clippy::must_use_candidate,
    clippy::needless_doctest_main,
    clippy::unreadable_literal
)]

extern crate creusot_std;

#[cfg(not(creusot))]
mod runtime;
#[cfg(not(creusot))]
pub use runtime::{Buffer, Integer};

#[cfg(creusot)]
mod verification;
#[cfg(creusot)]
pub use verification::{decimal_values, integer_decimal_values, Buffer, Integer};
