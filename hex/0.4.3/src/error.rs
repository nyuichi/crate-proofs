use core::fmt;
#[allow(unused_imports)]
use creusot_std::invariant::inv;
#[allow(unused_imports)]
use creusot_std::prelude::{ensures, logic, pearlite, trusted, DeepModel, Invariant, View};

/// The error type for decoding a hex string into `Vec<u8>` or `[u8; N]`.
#[cfg_attr(not(creusot), derive(Debug))]
#[derive(Clone, Copy, DeepModel)]
pub enum FromHexError {
    /// An invalid character was found. Valid ones are: `0...9`, `a...f`
    /// or `A...F`.
    InvalidHexCharacter { c: char, index: usize },

    /// A hex string's length needs to be even, as two digits correspond to
    /// one byte.
    OddLength,

    /// If the hex string is decoded into a fixed sized container, such as an
    /// array, the hex string's length * 2 has to match the container's
    /// length.
    InvalidStringLength,
}

// The ordinary build keeps the upstream derived representation. Creusot does
// not model mutation through `fmt::Formatter`, so the equivalent formatting
// body is an explicit integration boundary in the verification build.
#[cfg(creusot)]
impl fmt::Debug for FromHexError {
    #[trusted]
    #[ensures(creusot_std::std::fmt::formatter_extends(
        f.deep_model(),
        (^f).deep_model(),
    ))]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FromHexError::InvalidHexCharacter { c, index } => f
                .debug_struct("InvalidHexCharacter")
                .field("c", c)
                .field("index", index)
                .finish(),
            FromHexError::OddLength => f.write_str("OddLength"),
            FromHexError::InvalidStringLength => f.write_str("InvalidStringLength"),
        }
    }
}

impl PartialEq for FromHexError {
    #[ensures(result == (self.deep_model() == other.deep_model()))]
    fn eq(&self, other: &Self) -> bool {
        match (*self, *other) {
            (
                FromHexError::InvalidHexCharacter {
                    c: left_c,
                    index: left_index,
                },
                FromHexError::InvalidHexCharacter {
                    c: right_c,
                    index: right_index,
                },
            ) => left_c == right_c && left_index == right_index,
            (FromHexError::OddLength, FromHexError::OddLength) => true,
            (FromHexError::InvalidStringLength, FromHexError::InvalidStringLength) => true,
            _ => false,
        }
    }
}

impl View for FromHexError {
    type ViewTy = (u8, char, usize);

    /// Preserve the discriminant and public fields in a compact logical view.
    #[logic(open)]
    fn view(self) -> Self::ViewTy {
        match self {
            FromHexError::InvalidHexCharacter { c, index } => (0u8, c, index),
            FromHexError::OddLength => (1u8, '\0', 0usize),
            FromHexError::InvalidStringLength => (2u8, '\0', 0usize),
        }
    }
}

impl Invariant for FromHexError {
    /// All publicly constructible variants are valid values. Semantic facts
    /// such as "the character is invalid at this input index" cannot be a
    /// type invariant because callers may construct the public variant
    /// directly; decoder contracts establish those stronger provenance facts.
    #[logic(open, prophetic)]
    fn invariant(self) -> bool {
        pearlite! {
            match self {
                FromHexError::InvalidHexCharacter { c, index } => inv(c) && inv(index),
                FromHexError::OddLength => inv(()),
                FromHexError::InvalidStringLength => inv(()),
            }
        }
    }
}

#[cfg(feature = "std")]
impl std::error::Error for FromHexError {}

impl fmt::Display for FromHexError {
    // Formatting is delegated to core::fmt, whose Formatter mutation is not
    // modeled by Creusot. The decoder never relies on this implementation;
    // its exact strings remain covered by the upstream tests.
    #[trusted]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            FromHexError::InvalidHexCharacter { c, index } => {
                write!(f, "Invalid character {:?} at position {}", c, index)
            }
            FromHexError::OddLength => write!(f, "Odd number of digits"),
            FromHexError::InvalidStringLength => write!(f, "Invalid string length"),
        }
    }
}

#[cfg(test)]
// this feature flag is here to suppress unused
// warnings of `super::*` and `pretty_assertions::assert_eq`
#[cfg(feature = "alloc")]
mod tests {
    use super::*;
    #[cfg(feature = "alloc")]
    use alloc::string::ToString;
    use pretty_assertions::assert_eq;

    #[test]
    #[cfg(feature = "alloc")]
    fn test_display() {
        assert_eq!(
            FromHexError::InvalidHexCharacter { c: '\n', index: 5 }.to_string(),
            "Invalid character '\\n' at position 5"
        );

        assert_eq!(FromHexError::OddLength.to_string(), "Odd number of digits");
        assert_eq!(
            FromHexError::InvalidStringLength.to_string(),
            "Invalid string length"
        );
    }
}
