//! Hex encoding with `serde`.
#[cfg_attr(
    all(feature = "alloc", feature = "serde"),
    doc = r##"
# Example

```
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
struct Foo {
    #[serde(with = "hex")]
    bar: Vec<u8>,
}
```
"##
)]
#[cfg(not(creusot))]
use serde::de::{Error, Visitor};
use serde::Deserializer;
#[cfg(feature = "alloc")]
use serde::Serializer;

#[cfg(all(feature = "alloc", not(creusot)))]
use alloc::string::String;

use core::fmt;
#[cfg(not(creusot))]
use core::marker::PhantomData;

use crate::FromHex;

#[cfg(feature = "alloc")]
use crate::ToHex;

#[cfg(all(feature = "alloc", creusot))]
use crate::{is_lower_char_encoding, is_upper_char_encoding};
#[cfg(creusot)]
use creusot_std::prelude::{ensures, logic, trusted, Seq};

/// Logical relation supplied by a serde serializer for `serialize_str`.
#[cfg(all(feature = "alloc", creusot))]
#[trusted]
#[logic(opaque)]
pub fn serializer_accepts<S: Serializer>(
    _serializer: &S,
    _text: Seq<char>,
    _outcome: Result<S::Ok, S::Error>,
) -> bool {
    pearlite! { dead }
}

/// Logical relation between a deserializer and a string it supplies to the
/// visitor. Serde itself has no Creusot model, so this is an explicit boundary.
#[cfg(creusot)]
#[trusted]
#[logic(opaque)]
pub fn deserializer_yields<'de, D: Deserializer<'de>>(_deserializer: &D, _input: Seq<u8>) -> bool {
    pearlite! { dead }
}

/// Records a serde-layer failure before a successfully decoded value exists.
#[cfg(creusot)]
#[trusted]
#[logic(opaque)]
pub fn deserializer_rejects<'de, D: Deserializer<'de>>(_deserializer: &D) -> bool {
    pearlite! { dead }
}

/// Serializes `data` as hex string using uppercase characters.
///
/// Apart from the characters' casing, this works exactly like `serialize()`.
#[cfg(feature = "alloc")]
#[cfg(not(creusot))]
pub fn serialize_upper<S, T>(data: T, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
    T: ToHex,
{
    let s = data.encode_hex_upper::<String>();
    serializer.serialize_str(&s)
}

#[cfg(all(feature = "alloc", creusot))]
#[trusted]
#[ensures(exists<text: Seq<char>>
    is_upper_char_encoding(data.hex_bytes(), text)
        && serializer_accepts(&serializer, text, result)
)]
pub fn serialize_upper<S, T>(data: T, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
    T: ToHex,
{
    let _ = (data, serializer);
    panic!("verification-only serde boundary")
}

/// Serializes `data` as hex string using lowercase characters.
///
/// Lowercase characters are used (e.g. `f9b4ca`). The resulting string's length
/// is always even, each byte in data is always encoded using two hex digits.
/// Thus, the resulting string contains exactly twice as many bytes as the input
/// data.
#[cfg(feature = "alloc")]
#[cfg(not(creusot))]
pub fn serialize<S, T>(data: T, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
    T: ToHex,
{
    let s = data.encode_hex::<String>();
    serializer.serialize_str(&s)
}

#[cfg(all(feature = "alloc", creusot))]
#[trusted]
#[ensures(exists<text: Seq<char>>
    is_lower_char_encoding(data.hex_bytes(), text)
        && serializer_accepts(&serializer, text, result)
)]
pub fn serialize<S, T>(data: T, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
    T: ToHex,
{
    let _ = (data, serializer);
    panic!("verification-only serde boundary")
}

/// Deserializes a hex string into raw bytes.
///
/// Both, upper and lower case characters are valid in the input string and can
/// even be mixed (e.g. `f9b4ca`, `F9B4CA` and `f9B4Ca` are all valid strings).
#[cfg(not(creusot))]
pub fn deserialize<'de, D, T>(deserializer: D) -> Result<T, D::Error>
where
    D: Deserializer<'de>,
    T: FromHex,
    <T as FromHex>::Error: fmt::Display,
{
    struct HexStrVisitor<T>(PhantomData<T>);

    impl<'de, T> Visitor<'de> for HexStrVisitor<T>
    where
        T: FromHex,
        <T as FromHex>::Error: fmt::Display,
    {
        type Value = T;

        fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
            write!(f, "a hex encoded string")
        }

        fn visit_str<E>(self, data: &str) -> Result<Self::Value, E>
        where
            E: Error,
        {
            FromHex::from_hex(data).map_err(Error::custom)
        }

        fn visit_borrowed_str<E>(self, data: &'de str) -> Result<Self::Value, E>
        where
            E: Error,
        {
            FromHex::from_hex(data).map_err(Error::custom)
        }
    }

    deserializer.deserialize_str(HexStrVisitor(PhantomData))
}

#[cfg(creusot)]
#[trusted]
#[ensures(match result {
    Ok(value) => exists<input: Seq<u8>>
        deserializer_yields(&deserializer, input)
            && T::from_hex_post(input, Ok(value)),
    Err(_) => deserializer_rejects(&deserializer),
})]
pub fn deserialize<'de, D, T>(deserializer: D) -> Result<T, D::Error>
where
    D: Deserializer<'de>,
    T: FromHex,
    <T as FromHex>::Error: fmt::Display,
{
    let _ = deserializer;
    panic!("verification-only serde boundary")
}
