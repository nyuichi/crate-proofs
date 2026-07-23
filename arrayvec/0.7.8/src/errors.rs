use std::fmt;
#[allow(unused_imports)]
use creusot_std::prelude::trusted;
#[cfg(feature="std")]
use std::any::Any;
#[cfg(feature="std")]
use std::error::Error;

/// Error value indicating insufficient capacity
#[cfg_attr(not(creusot), derive(Clone, Copy, Eq, Ord, PartialEq, PartialOrd))]
#[cfg_attr(creusot, derive(Clone, Copy))]
pub struct CapacityError<T = ()> {
    element: T,
}

impl<T> CapacityError<T> {
    /// Create a new `CapacityError` from `element`.
    pub const fn new(element: T) -> CapacityError<T> {
        CapacityError {
            element: element,
        }
    }

    /// Extract the overflowing element
    pub fn element(self) -> T {
        self.element
    }

    /// Convert into a `CapacityError` that does not carry an element.
    pub fn simplify(self) -> CapacityError {
        CapacityError { element: () }
    }
}

const CAPERROR: &'static str = "insufficient capacity";

#[cfg(all(feature="std", not(creusot)))]
/// Requires `features="std"`.
impl<T: Any> Error for CapacityError<T> {}

#[cfg(not(creusot))]
impl<T> fmt::Display for CapacityError<T> {
    #[trusted]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", CAPERROR)
    }
}

#[cfg(not(creusot))]
impl<T> fmt::Debug for CapacityError<T> {
    #[trusted]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}: {}", "CapacityError", CAPERROR)
    }
}
