use crate::prelude::*;
use alloc::borrow::Cow;

impl View for Cow<'_, [u8]> {
    type ViewTy = Seq<u8>;

    #[logic(open)]
    fn view(self) -> Self::ViewTy {
        match self {
            Cow::Borrowed(bytes) => pearlite! { bytes@ },
            Cow::Owned(bytes) => pearlite! { bytes@ },
        }
    }
}

impl DeepModel for Cow<'_, [u8]> {
    type DeepModelTy = Seq<Int>;

    #[logic]
    fn deep_model(self) -> Self::DeepModelTy {
        pearlite! { self@.map(|byte: u8| byte@) }
    }
}

#[cfg(feature = "std")]
impl View for Cow<'_, str> {
    type ViewTy = Seq<char>;

    #[logic(open)]
    fn view(self) -> Self::ViewTy {
        match self {
            Cow::Borrowed(value) => pearlite! { value@ },
            Cow::Owned(value) => pearlite! { value@ },
        }
    }
}

#[cfg(feature = "std")]
impl DeepModel for Cow<'_, str> {
    type DeepModelTy = Seq<Int>;

    #[logic]
    fn deep_model(self) -> Self::DeepModelTy {
        pearlite! { self@.map(|character: char| character@) }
    }
}

pub trait CowBytesExt {
    #[logic]
    fn is_borrowed(self) -> bool;
}

impl CowBytesExt for Cow<'_, [u8]> {
    #[logic(open)]
    fn is_borrowed(self) -> bool {
        match self {
            Cow::Borrowed(_) => true,
            Cow::Owned(_) => false,
        }
    }
}

#[cfg(feature = "std")]
pub trait CowStrExt {
    #[logic]
    fn is_borrowed(self) -> bool;
}

#[cfg(feature = "std")]
impl CowStrExt for Cow<'_, str> {
    #[logic(open)]
    fn is_borrowed(self) -> bool {
        match self {
            Cow::Borrowed(_) => true,
            Cow::Owned(_) => false,
        }
    }
}
