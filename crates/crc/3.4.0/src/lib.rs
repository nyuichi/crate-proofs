//! # crc
//! Rust implementation of CRC.
//!
//! ### Examples
//! Using a well-known algorithm:
//! ```rust
//! const X25: crc::Crc<u16> = crc::Crc::<u16>::new(&crc::CRC_16_IBM_SDLC);
//! assert_eq!(X25.checksum(b"123456789"), 0x906e);
//! ```
//!
//! Using a custom algorithm:
//! ```rust
//! const CUSTOM_ALG: crc::Algorithm<u16> = crc::Algorithm {
//!     width: 16,
//!     poly: 0x8005,
//!     init: 0xffff,
//!     refin: false,
//!     refout: false,
//!     xorout: 0x0000,
//!     check: 0xaee7,
//!     residue: 0x0000
//! };
//! let crc = crc::Crc::<u16>::new(&CUSTOM_ALG);
//! let mut digest = crc.digest();
//! digest.update(b"123456789");
//! assert_eq!(digest.finalize(), 0xaee7);
//! ```
#![no_std]
#![forbid(unsafe_code)]

extern crate creusot_std;

#[allow(unused_imports)]
use creusot_std::prelude::{
    ensures, invariant, logic, pearlite, requires, trusted, Invariant, Seq, View,
};

pub use crc_catalog::algorithm::*;
pub use crc_catalog::{Algorithm, Width};

mod crc128;
mod crc16;
mod crc32;
mod crc64;
mod crc8;
mod model;
mod table;
mod util;

#[allow(unused_imports)]
pub use model::*;

/// A trait for CRC implementations.
pub trait Implementation {
    /// Associated data necessary for the implementation (e.g. lookup tables).
    type Data<W>;
}

/// A table-based implementation of the CRC algorithm, with `L` lanes.
/// The number of entries in the lookup table is `L * 256`.
pub struct Table<const L: usize> {}

impl<const L: usize> Copy for Table<L> {}

impl<const L: usize> Clone for Table<L> {
    #[ensures(result.invariant())]
    fn clone(&self) -> Self {
        *self
    }
}

impl<const L: usize> Invariant for Table<L> {
    /// `Table` is a zero-sized implementation marker; every value is valid.
    #[logic(open)]
    fn invariant(self) -> bool {
        pearlite! { true }
    }
}

/// An implementation of the CRC algorithm with no lookup table.
pub type NoTable = Table<0>;

type DefaultImpl = Table<1>;

impl<const L: usize> Implementation for Table<L> {
    type Data<W> = [[W; 256]; L];
}

mod private {
    pub trait Sealed {}
    impl Sealed for super::Table<0> {}
    impl Sealed for super::Table<1> {}
    impl Sealed for super::Table<16> {}
}

/// Crc instance with a specific width, algorithm, and implementation.
pub struct Crc<W: Width, I: Implementation = DefaultImpl> {
    pub algorithm: &'static Algorithm<W>,
    data: I::Data<W>,
}

impl<W: Width, I: Implementation> Crc<W, I> {
    /// Opaque logical observer for the implementation data.
    #[logic]
    pub fn implementation_data(self) -> I::Data<W> {
        self.data
    }
}

impl<W: Width, I: Implementation> Clone for Crc<W, I>
where
    I::Data<W>: Clone,
{
    #[trusted]
    #[ensures(result.algorithm == self.algorithm)]
    #[ensures(result.implementation_data() == self.implementation_data())]
    fn clone(&self) -> Self {
        Self {
            algorithm: self.algorithm,
            data: self.data.clone(),
        }
    }
}

pub struct Digest<'a, W: Width, I: Implementation = DefaultImpl> {
    crc: &'a Crc<W, I>,
    value: W,
}

impl<'a, W: Width, I: Implementation> Digest<'a, W, I> {
    /// Opaque logical observer for the borrowed CRC configuration.
    #[logic]
    pub fn crc_model(self) -> &'a Crc<W, I> {
        self.crc
    }

    /// Opaque logical observer for the unfinalized register.
    #[logic]
    pub fn state_model(self) -> W {
        self.value
    }
}

impl<'a, W: Width, I: Implementation> Clone for Digest<'a, W, I>
where
    W: Clone,
{
    #[trusted]
    #[ensures(result.crc_model() == self.crc_model())]
    #[ensures(result.state_model() == self.state_model())]
    fn clone(&self) -> Self {
        Self {
            crc: self.crc,
            value: self.value.clone(),
        }
    }
}

macro_rules! impl_crc_models {
    ($word:ty, $valid:ident, $table_valid:ident) => {
        impl<const L: usize> Invariant for Crc<$word, Table<L>> {
            /// The algorithm width fits the register and every lookup lane is
            /// generated from the table-independent byte recurrence.
            #[logic]
            fn invariant(self) -> bool {
                pearlite! {
                    $valid(self.algorithm.width)
                        && $table_valid(
                            self.algorithm.width,
                            self.algorithm.poly,
                            self.algorithm.refin,
                            self.data,
                        )
                }
            }
        }

        impl<const L: usize> Crc<$word, Table<L>> {
            /// Logical view of the generated lookup lanes exposed by `table`.
            #[logic]
            pub fn table_model(self) -> Seq<[$word; 256]> {
                pearlite! { self.data@ }
            }
        }

        impl<'a, const L: usize> View for Digest<'a, $word, Table<L>> {
            type ViewTy = $word;

            /// The current unfinalized, aligned CRC register.
            #[logic]
            fn view(self) -> Self::ViewTy {
                self.value
            }
        }

        impl<'a, const L: usize> Digest<'a, $word, Table<L>> {
            /// Algorithm associated with this in-progress digest.
            #[logic]
            pub fn algorithm(self) -> &'static Algorithm<$word> {
                self.crc.algorithm
            }
        }

        impl<'a, const L: usize> Invariant for Digest<'a, $word, Table<L>> {
            /// A digest is valid exactly when its borrowed CRC configuration
            /// has a valid width and matching generated tables.
            #[logic]
            fn invariant(self) -> bool {
                pearlite! { (*self.crc).invariant() }
            }
        }
    };
}

impl_crc_models!(u8, valid_width_8, crc8_table_valid);
impl_crc_models!(u16, valid_width_16, crc16_table_valid);
impl_crc_models!(u32, valid_width_32, crc32_table_valid);
impl_crc_models!(u64, valid_width_64, crc64_table_valid);
impl_crc_models!(u128, valid_width_128, crc128_table_valid);

#[cfg(test)]
mod test {
    use super::{Crc, CRC_32_ISCSI};

    #[test]
    fn test_clone() {
        const CRC: Crc<u32> = Crc::<u32>::new(&CRC_32_ISCSI);
        let crc = CRC.clone();
        let digest = crc.digest();
        let _digest = digest.clone();
    }
}
