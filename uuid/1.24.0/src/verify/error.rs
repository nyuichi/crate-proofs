#![allow(missing_docs)]

#[allow(missing_debug_implementations)]
pub struct Error(pub(crate) ErrorKind);

pub(crate) enum ErrorKind {
    ParseByteLength { len: usize },
    ParseOther,
    Nil,
}
