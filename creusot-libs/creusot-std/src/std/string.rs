use crate::prelude::*;
#[cfg(all(creusot, feature = "std"))]
use crate::std::cow::CowStrExt;
#[cfg(all(creusot, feature = "std"))]
use alloc::string::FromUtf8Error;
#[cfg(all(creusot, feature = "std"))]
use core::ops::Deref;
#[cfg(creusot)]
use core::str::Utf8Error;

/// Whether a byte sequence is exactly the UTF-8 encoding of characters.
#[logic(open)]
pub fn valid_utf8(bytes: Seq<u8>) -> bool {
    pearlite! { exists<characters: Seq<char>> characters.to_bytes() == bytes }
}

/// Rust's complete lossy UTF-8 conversion, including U+FFFD placement.
/// This opaque observer is a deliberate standard-library trust boundary.
#[trusted]
#[logic(opaque)]
#[ensures(valid_utf8(bytes) ==> result.to_bytes() == bytes)]
pub fn utf8_lossy_model(bytes: Seq<u8>) -> Seq<char> {
    dead
}

/// Connect invalid input with the observable fields of `Utf8Error`.
#[trusted]
#[logic(opaque)]
#[ensures(result ==> 0 <= valid_up_to && valid_up_to <= bytes.len())]
#[ensures(result ==> valid_utf8(bytes.subsequence(0, valid_up_to)))]
#[ensures(result ==> !valid_utf8(bytes))]
#[ensures(result ==> match error_len {
    Some(length) => 1 <= length && length <= 3 && valid_up_to + length <= bytes.len(),
    None => valid_up_to < bytes.len() && bytes.len() - valid_up_to < 4,
})]
pub fn utf8_error_matches(bytes: Seq<u8>, valid_up_to: Int, error_len: Option<Int>) -> bool {
    dead
}

pub trait Utf8ErrorExt {
    #[logic]
    fn valid_up_to_logic(self) -> Int;

    #[logic]
    fn error_len_logic(self) -> Option<Int>;
}

impl Utf8ErrorExt for core::str::Utf8Error {
    #[trusted]
    #[logic(opaque)]
    fn valid_up_to_logic(self) -> Int {
        dead
    }

    #[trusted]
    #[logic(opaque)]
    fn error_len_logic(self) -> Option<Int> {
        dead
    }
}

#[cfg(feature = "std")]
pub trait FromUtf8ErrorExt {
    #[logic]
    fn utf8_error_logic(self) -> core::str::Utf8Error;
}

#[cfg(feature = "std")]
impl FromUtf8ErrorExt for alloc::string::FromUtf8Error {
    #[trusted]
    #[logic(opaque)]
    fn utf8_error_logic(self) -> core::str::Utf8Error {
        dead
    }
}

impl View for str {
    type ViewTy = Seq<char>;

    #[logic(opaque)]
    fn view(self) -> Self::ViewTy {
        dead
    }
}

impl DeepModel for str {
    type DeepModelTy = Seq<char>;

    #[logic]
    fn deep_model(self) -> Self::DeepModelTy {
        self.view()
    }
}

#[cfg(feature = "std")]
impl View for String {
    type ViewTy = Seq<char>;

    #[logic(opaque)]
    fn view(self) -> Self::ViewTy {
        dead
    }
}

#[cfg(feature = "std")]
impl DeepModel for String {
    type DeepModelTy = Seq<char>;

    #[logic]
    fn deep_model(self) -> Self::DeepModelTy {
        self.view()
    }
}

#[cfg(feature = "std")]
impl FromIteratorSpec<char> for String {
    #[logic(open)]
    fn from_iter_post(produced: Seq<char>, value: Self) -> bool {
        pearlite! { value@ == produced }
    }
}

#[cfg(feature = "std")]
extern_spec! {
    impl Deref for String {
        #[check(ghost)]
        #[ensures(result@ == self@)]
        fn deref(&self) -> &str;
    }

    impl String {
        #[check(ghost)]
        #[ensures(result@ == self@.to_bytes().len())]
        fn len(&self) -> usize;

        #[ensures((^self)@ == self@.concat(value@))]
        fn push_str(&mut self, value: &str);

        #[ensures(match result {
            Ok(value) => value@.to_bytes() == bytes@,
            Err(error) => utf8_error_matches(
                bytes@,
                error.utf8_error_logic().valid_up_to_logic(),
                error.utf8_error_logic().error_len_logic(),
            ),
        })]
        fn from_utf8(bytes: Vec<u8>) -> Result<String, FromUtf8Error>;

        #[ensures(result@ == utf8_lossy_model(bytes@))]
        #[ensures(result.is_borrowed() == valid_utf8(bytes@))]
        fn from_utf8_lossy(bytes: &[u8]) -> alloc::borrow::Cow<'_, str>;

        #[check(ghost)]
        #[requires(exists<s: Seq<char>> s.to_bytes() == bytes@)]
        #[ensures(result@.to_bytes() == bytes@)]
        #[ensures(result@ == utf8_lossy_model(bytes@))]
        unsafe fn from_utf8_unchecked(bytes: Vec<u8>) -> String;
    }

    impl FromUtf8Error {
        #[check(ghost)]
        #[ensures(result.valid_up_to_logic() == self.utf8_error_logic().valid_up_to_logic())]
        #[ensures(result.error_len_logic() == self.utf8_error_logic().error_len_logic())]
        fn utf8_error(&self) -> core::str::Utf8Error;
    }
}

extern_spec! {
    impl str {
        #[check(ghost)]
        #[ensures(result@ == self@.to_bytes())]
        fn as_bytes(&self) -> &[u8];

        #[check(ghost)]
        #[ensures(result@ == self@.to_bytes().len())]
        fn len(&self) -> usize;

        #[check(ghost)]
        #[requires(exists<i0> 0 <= i0 && i0 <= self@.len() && self@.subsequence(0, i0).to_bytes().len() == ix@)]
        #[ensures(result.0@.concat(result.1@) == self@)]
        #[ensures(result.0@.to_bytes().len() == ix@)]
        fn split_at(&self, ix: usize) -> (&str, &str);
    }

    mod core {
        mod str {
            #[ensures(match result {
                Ok(value) => value@.to_bytes() == bytes@,
                Err(error) => utf8_error_matches(
                    bytes@,
                    error.valid_up_to_logic(),
                    error.error_len_logic(),
                ),
            })]
            fn from_utf8(bytes: &[u8]) -> Result<&str, Utf8Error>;

            #[requires(exists<s: Seq<char>> s.to_bytes() == bytes@)]
            #[ensures(result@.to_bytes() == bytes@)]
            unsafe fn from_utf8_unchecked(bytes: &[u8]) -> &str;

        }
    }

    impl core::str::Utf8Error {
        #[check(ghost)]
        #[ensures(result@ == self.valid_up_to_logic())]
        fn valid_up_to(&self) -> usize;

        #[check(ghost)]
        #[ensures(result.deep_model() == self.error_len_logic())]
        fn error_len(&self) -> Option<usize>;
    }
}

#[cfg(feature = "std")]
extern_spec! {
    impl Clone for Box<str> {
        #[check(ghost)]
        #[ensures((*result)@ == (**self)@)]
        fn clone(&self) -> Box<str>;
    }

    impl ToOwned for str {
        #[check(terminates)] // can OOM (?)
        #[ensures(result@ == self@)]
        fn to_owned(&self) -> String;
    }
}

impl Seq<char> {
    #[logic(open)]
    pub fn to_bytes(self) -> Seq<u8> {
        pearlite! { self.flat_map(|c: char| c.to_utf8()) }
    }
}

#[trusted]
#[logic(open)]
#[ensures(forall<s1: Seq<char>, s2: Seq<char>> s1.to_bytes() == s2.to_bytes() ==> s1 == s2)]
pub fn injective_to_bytes() {}
