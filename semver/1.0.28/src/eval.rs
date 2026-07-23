use crate::{Comparator, Op, Version, VersionReq};
use core::cmp::Ordering;
#[allow(unused_imports)]
use creusot_std::prelude::{ensures, invariant, logic, pearlite, trusted, Int};

/// Logical observation of whether a prerelease identifier is empty.
///
/// This is opaque because the runtime `Identifier` uses a pointer-tagged,
/// short-string-optimized representation.
#[logic(opaque)]
#[allow(unused_variables)]
pub fn prerelease_is_empty_model(pre: &crate::Prerelease) -> bool {
    pearlite! { dead }
}

/// Logical SemVer precedence comparison of two prerelease identifiers.
///
/// The runtime implementation of this ordering remains in `impls.rs`; this
/// observer is the single proof boundary through which evaluation uses it.
#[logic(opaque)]
#[allow(unused_variables)]
pub fn prerelease_cmp_model(left: &crate::Prerelease, right: &crate::Prerelease) -> Ordering {
    pearlite! { dead }
}

#[trusted]
#[ensures(result == prerelease_cmp_model(left, right))]
fn prerelease_cmp(left: &crate::Prerelease, right: &crate::Prerelease) -> Ordering {
    #[cfg(not(creusot))]
    {
        left.cmp(right)
    }
    #[cfg(creusot)]
    {
        let _ = (left, right);
        Ordering::Equal
    }
}

#[logic(open)]
pub fn matches_exact_model(cmp: &Comparator, ver: &Version) -> bool {
    pearlite! {
        ver.major == cmp.major
            && (match cmp.minor { None => true, Some(minor) => ver.minor == minor })
            && (match cmp.patch { None => true, Some(patch) => ver.patch == patch })
            && prerelease_cmp_model(&ver.pre, &cmp.pre) == Ordering::Equal
    }
}

#[logic(open)]
pub fn matches_greater_model(cmp: &Comparator, ver: &Version) -> bool {
    pearlite! {
        if ver.major != cmp.major {
            ver.major > cmp.major
        } else {
            match cmp.minor {
                None => false,
                Some(minor) => if ver.minor != minor {
                    ver.minor > minor
                } else {
                    match cmp.patch {
                        None => false,
                        Some(patch) => if ver.patch != patch {
                            ver.patch > patch
                        } else {
                            prerelease_cmp_model(&ver.pre, &cmp.pre) == Ordering::Greater
                        }
                    }
                }
            }
        }
    }
}

#[logic(open)]
pub fn matches_less_model(cmp: &Comparator, ver: &Version) -> bool {
    pearlite! {
        if ver.major != cmp.major {
            ver.major < cmp.major
        } else {
            match cmp.minor {
                None => false,
                Some(minor) => if ver.minor != minor {
                    ver.minor < minor
                } else {
                    match cmp.patch {
                        None => false,
                        Some(patch) => if ver.patch != patch {
                            ver.patch < patch
                        } else {
                            prerelease_cmp_model(&ver.pre, &cmp.pre) == Ordering::Less
                        }
                    }
                }
            }
        }
    }
}

#[logic(open)]
pub fn matches_tilde_model(cmp: &Comparator, ver: &Version) -> bool {
    pearlite! {
        ver.major == cmp.major
            && (match cmp.minor { None => true, Some(minor) => ver.minor == minor })
            && (match cmp.patch {
                None => prerelease_cmp_model(&ver.pre, &cmp.pre) != Ordering::Less,
                Some(patch) => ver.patch > patch
                    || (ver.patch == patch
                        && prerelease_cmp_model(&ver.pre, &cmp.pre) != Ordering::Less),
            })
    }
}

#[logic(open)]
pub fn matches_caret_model(cmp: &Comparator, ver: &Version) -> bool {
    pearlite! {
        if ver.major != cmp.major {
            false
        } else {
            match cmp.minor {
                None => true,
                Some(minor) => match cmp.patch {
                    None => if cmp.major > 0u64 { ver.minor >= minor } else { ver.minor == minor },
                    Some(patch) => {
                        let numeric_match = if cmp.major > 0u64 {
                            ver.minor > minor || (ver.minor == minor && ver.patch >= patch)
                        } else if minor > 0u64 {
                            ver.minor == minor && ver.patch >= patch
                        } else {
                            ver.minor == minor && ver.patch == patch
                        };
                        numeric_match
                            && (ver.minor != minor
                                || ver.patch != patch
                                || prerelease_cmp_model(&ver.pre, &cmp.pre) != Ordering::Less)
                    }
                }
            }
        }
    }
}

#[logic(open)]
pub fn matches_impl_model(cmp: &Comparator, ver: &Version) -> bool {
    match &cmp.op {
        Op::Exact | Op::Wildcard => matches_exact_model(cmp, ver),
        Op::Greater => matches_greater_model(cmp, ver),
        Op::GreaterEq => matches_exact_model(cmp, ver) || matches_greater_model(cmp, ver),
        Op::Less => matches_less_model(cmp, ver),
        Op::LessEq => matches_exact_model(cmp, ver) || matches_less_model(cmp, ver),
        Op::Tilde => matches_tilde_model(cmp, ver),
        Op::Caret => matches_caret_model(cmp, ver),
    }
}

#[logic(open)]
pub fn pre_is_compatible_model(cmp: &Comparator, ver: &Version) -> bool {
    pearlite! {
        cmp.major == ver.major
            && cmp.minor == Some(ver.minor)
            && cmp.patch == Some(ver.patch)
            && !prerelease_is_empty_model(&cmp.pre)
    }
}

#[logic(open)]
pub fn matches_req_model(req: &VersionReq, ver: &Version) -> bool {
    pearlite! {
        (forall<i: Int> 0 <= i && i < req.comparators@.len()
            ==> matches_impl_model(&req.comparators@[i], ver))
        && (prerelease_is_empty_model(&ver.pre)
            || exists<i: Int> 0 <= i && i < req.comparators@.len()
                && pre_is_compatible_model(&req.comparators@[i], ver))
    }
}

#[ensures(result == matches_req_model(req, ver))]
pub(crate) fn matches_req(req: &VersionReq, ver: &Version) -> bool {
    let mut i = 0usize;
    #[invariant(i@ <= req.comparators@.len())]
    #[invariant(forall<j: Int> 0 <= j && j < i@
        ==> matches_impl_model(&req.comparators@[j], ver))]
    while i < req.comparators.len() {
        if !matches_impl(&req.comparators[i], ver) {
            return false;
        }
        i += 1;
    }

    if ver.pre.is_empty() {
        return true;
    }

    // If a version has a prerelease tag (for example, 1.2.3-alpha.3) then it
    // will only be allowed to satisfy req if at least one comparator with the
    // same major.minor.patch also has a prerelease tag.
    let mut i = 0usize;
    #[invariant(i@ <= req.comparators@.len())]
    #[invariant(forall<j: Int> 0 <= j && j < i@
        ==> !pre_is_compatible_model(&req.comparators@[j], ver))]
    while i < req.comparators.len() {
        if pre_is_compatible(&req.comparators[i], ver) {
            return true;
        }
        i += 1;
    }

    false
}

#[ensures(result == (matches_impl_model(cmp, ver)
    && (prerelease_is_empty_model(&ver.pre) || pre_is_compatible_model(cmp, ver))))]
pub(crate) fn matches_comparator(cmp: &Comparator, ver: &Version) -> bool {
    matches_impl(cmp, ver) && (ver.pre.is_empty() || pre_is_compatible(cmp, ver))
}

#[ensures(result == matches_impl_model(cmp, ver))]
fn matches_impl(cmp: &Comparator, ver: &Version) -> bool {
    match &cmp.op {
        Op::Exact | Op::Wildcard => matches_exact(cmp, ver),
        Op::Greater => matches_greater(cmp, ver),
        Op::GreaterEq => matches_exact(cmp, ver) || matches_greater(cmp, ver),
        Op::Less => matches_less(cmp, ver),
        Op::LessEq => matches_exact(cmp, ver) || matches_less(cmp, ver),
        Op::Tilde => matches_tilde(cmp, ver),
        Op::Caret => matches_caret(cmp, ver),
    }
}

#[ensures(result == matches_exact_model(cmp, ver))]
fn matches_exact(cmp: &Comparator, ver: &Version) -> bool {
    if ver.major != cmp.major {
        return false;
    }

    if let Some(minor) = cmp.minor {
        if ver.minor != minor {
            return false;
        }
    }

    if let Some(patch) = cmp.patch {
        if ver.patch != patch {
            return false;
        }
    }

    prerelease_cmp(&ver.pre, &cmp.pre) == Ordering::Equal
}

#[ensures(result == matches_greater_model(cmp, ver))]
fn matches_greater(cmp: &Comparator, ver: &Version) -> bool {
    if ver.major != cmp.major {
        return ver.major > cmp.major;
    }

    match cmp.minor {
        None => return false,
        Some(minor) => {
            if ver.minor != minor {
                return ver.minor > minor;
            }
        }
    }

    match cmp.patch {
        None => return false,
        Some(patch) => {
            if ver.patch != patch {
                return ver.patch > patch;
            }
        }
    }

    prerelease_cmp(&ver.pre, &cmp.pre) == Ordering::Greater
}

#[ensures(result == matches_less_model(cmp, ver))]
fn matches_less(cmp: &Comparator, ver: &Version) -> bool {
    if ver.major != cmp.major {
        return ver.major < cmp.major;
    }

    match cmp.minor {
        None => return false,
        Some(minor) => {
            if ver.minor != minor {
                return ver.minor < minor;
            }
        }
    }

    match cmp.patch {
        None => return false,
        Some(patch) => {
            if ver.patch != patch {
                return ver.patch < patch;
            }
        }
    }

    prerelease_cmp(&ver.pre, &cmp.pre) == Ordering::Less
}

#[ensures(result == matches_tilde_model(cmp, ver))]
fn matches_tilde(cmp: &Comparator, ver: &Version) -> bool {
    if ver.major != cmp.major {
        return false;
    }

    if let Some(minor) = cmp.minor {
        if ver.minor != minor {
            return false;
        }
    }

    if let Some(patch) = cmp.patch {
        if ver.patch != patch {
            return ver.patch > patch;
        }
    }

    prerelease_cmp(&ver.pre, &cmp.pre) != Ordering::Less
}

#[ensures(result == matches_caret_model(cmp, ver))]
fn matches_caret(cmp: &Comparator, ver: &Version) -> bool {
    if ver.major != cmp.major {
        return false;
    }

    let Some(minor) = cmp.minor else {
        return true;
    };

    let Some(patch) = cmp.patch else {
        if cmp.major > 0 {
            return ver.minor >= minor;
        } else {
            return ver.minor == minor;
        }
    };

    if cmp.major > 0 {
        if ver.minor != minor {
            return ver.minor > minor;
        } else if ver.patch != patch {
            return ver.patch > patch;
        }
    } else if minor > 0 {
        if ver.minor != minor {
            return false;
        } else if ver.patch != patch {
            return ver.patch > patch;
        }
    } else if ver.minor != minor || ver.patch != patch {
        return false;
    }

    prerelease_cmp(&ver.pre, &cmp.pre) != Ordering::Less
}

#[ensures(result == pre_is_compatible_model(cmp, ver))]
fn pre_is_compatible(cmp: &Comparator, ver: &Version) -> bool {
    cmp.major == ver.major
        && cmp.minor == Some(ver.minor)
        && cmp.patch == Some(ver.patch)
        && !cmp.pre.is_empty()
}
