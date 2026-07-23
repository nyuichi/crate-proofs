//! Creusot-facing scalar models for the crate's optimized byte searches.
//!
//! The runtime implementations use raw pointers, word-at-a-time loads and,
//! for ASCII scanning on x86-64, SSE2. This module isolates their functional
//! contracts from those representation details.

use creusot_std::prelude::{ensures, invariant, Int};

/// Return the first index containing a non-ASCII byte, or the slice length.
#[ensures(result@ <= slice@.len())]
#[ensures(forall<i: Int> 0 <= i && i < result@ ==> slice@[i]@ <= 0x7f)]
#[ensures(result@ < slice@.len() ==> slice@[result@]@ > 0x7f)]
pub fn first_non_ascii_byte(slice: &[u8]) -> usize {
    let mut i = 0;
    #[invariant(i@ <= slice@.len())]
    #[invariant(forall<j: Int> 0 <= j && j < i@ ==> slice@[j]@ <= 0x7f)]
    while i < slice.len() {
        if slice[i] > 0x7f {
            return i;
        }
        i += 1;
    }
    i
}

/// Return the first index whose byte differs from `needle`.
#[ensures(match result {
    Some(i) => i@ < haystack@.len()
        && haystack@[i@] != needle
        && (forall<j: Int> 0 <= j && j < i@ ==> haystack@[j] == needle),
    None => forall<j: Int> 0 <= j && j < haystack@.len() ==> haystack@[j] == needle,
})]
pub fn inv_memchr(needle: u8, haystack: &[u8]) -> Option<usize> {
    let mut i = 0;
    #[invariant(i@ <= haystack@.len())]
    #[invariant(forall<j: Int> 0 <= j && j < i@ ==> haystack@[j] == needle)]
    while i < haystack.len() {
        if haystack[i] != needle {
            return Some(i);
        }
        i += 1;
    }
    None
}

/// Return the last index whose byte differs from `needle`.
#[ensures(match result {
    Some(i) => i@ < haystack@.len()
        && haystack@[i@] != needle
        && (forall<j: Int> i@ < j && j < haystack@.len() ==> haystack@[j] == needle),
    None => forall<j: Int> 0 <= j && j < haystack@.len() ==> haystack@[j] == needle,
})]
pub fn inv_memrchr(needle: u8, haystack: &[u8]) -> Option<usize> {
    let mut i = haystack.len();
    #[invariant(i@ <= haystack@.len())]
    #[invariant(forall<j: Int> i@ <= j && j < haystack@.len() ==> haystack@[j] == needle)]
    while i > 0 {
        i -= 1;
        if haystack[i] != needle {
            return Some(i);
        }
    }
    None
}
